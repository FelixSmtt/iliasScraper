use colored::*;
use dashmap::DashMap;
use reqwest::Client;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::course::Course;

use crate::scraper::scrapable::Scrapeable;
use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;

pub async fn scrape_courses<'a>(
    client: &Client,
    courses: Vec<&'a Course>,
    opts: &ScrapeOptions,
) -> Vec<(&'a Course, ScrapeObject)> {
    let roots: Vec<Box<dyn Scrapeable>> = courses
        .iter()
        .enumerate()
        .map(|course| course.1.build_remote_root(course.0))
        .collect();

    let output = format!(
        ">> Scraping courses: {}",
        courses
            .iter()
            .map(|course| { course.name.clone() })
            .collect::<Vec<String>>()
            .join(", ")
    )
    .color(Color::Blue);
    println!("{}", output);
    stdout().flush().unwrap_or_default();

    let scraped_roots = scrape_trees(client.clone(), roots, opts.clone()).await;

    scraped_roots
        .into_iter()
        .filter_map(|root| {
            let fitting_course = courses
                .iter()
                .find(|course| course.get_root_url().eq(&root.url));
            if let Some(course) = fitting_course {
                return Some((*course, root));
            }
            None
        })
        .collect()
}

async fn scrape_trees(
    client: Client,
    roots: Vec<Box<dyn Scrapeable>>,
    opts: ScrapeOptions,
) -> Vec<ScrapeObject> {
    let start = Instant::now();

    let nodes = scrape_all_nodes(client, roots, opts).await;
    let tree = build_tree(nodes);

    let duration = start.elapsed();
    println!(" ({:?})", duration);

    tree
}

async fn scrape_all_nodes(
    client: Client,
    roots: Vec<Box<dyn Scrapeable>>,
    opts: ScrapeOptions,
) -> DashMap<Uuid, ScrapeObject> {
    let nodes = Arc::new(DashMap::new());
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let inflight = Arc::new(AtomicUsize::new(0));
    let opts = Arc::new(opts);
    let client = Arc::new(client);

    for root in roots {
        inflight.fetch_add(1, Ordering::SeqCst);
        tx.send(root).unwrap();
    }

    let mut tasks = tokio::task::JoinSet::new();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(10)); // tune concurrency

    loop {
        match rx.try_recv() {
            Ok(item) => {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let tx = tx.clone();
                let nodes = nodes.clone();
                let inflight = inflight.clone();
                let client = client.clone();
                let opts = opts.clone();

                tasks.spawn(async move {
                    let _permit = permit;

                    match item.scrape(&client, &opts).await {
                        Ok((result, children)) => {
                            nodes.insert(result.id, result);
                            let child_count = children.len();

                            if child_count > 0 {
                                inflight.fetch_add(child_count, Ordering::SeqCst);
                                for child in children {
                                    tx.send(child).unwrap();
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error scraping {}: {:?}", item.get_url(), e);
                        }
                    }

                    inflight.fetch_sub(1, Ordering::SeqCst);
                });
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                if inflight.load(Ordering::SeqCst) == 0 {
                    break;
                }
                // Sleep briefly to avoid busy-waiting
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                // Channel closed, but shouldn't happen here
                break;
            }
        }
    }

    while inflight.load(Ordering::SeqCst) > 0 {
        if let Some(res) = tasks.join_next().await {
            res.unwrap();
        }
    }
    Arc::unwrap_or_clone(nodes)
}

pub fn build_tree(nodes: DashMap<Uuid, ScrapeObject>) -> Vec<ScrapeObject> {
    let ids: Vec<Uuid> = nodes.iter().map(|n| *n.key()).collect();
    let parent_map: DashMap<Uuid, Vec<Uuid>> = DashMap::new();

    // Build map of parent with all children
    ids.iter().for_each(|id| {
        let node = nodes.get(id).unwrap();
        let parent = node.parent;

        if let Some(parent) = parent {
            if let Some(mut children) = parent_map.get_mut(&parent) {
                children.push(node.id);
            } else {
                parent_map.insert(parent, vec![node.id]);
            }
        }

        if !parent_map.contains_key(id) {
            parent_map.insert(*id, vec![]);
        }
    });

    // Go to each object that has not children in the parent_map and add it to the tree
    while !parent_map.is_empty() {
        let leaf_nodes: Vec<Uuid> = parent_map
            .iter()
            .filter_map(|pair| {
                if pair.value().is_empty() {
                    return Some(*pair.key());
                }

                None
            })
            .collect();

        for id in leaf_nodes {
            parent_map.remove(&id);

            let node = nodes.remove(&id).unwrap();
            let parent = node.1.parent;

            if let Some(parent) = parent {
                parent_map
                    .get_mut(&parent)
                    .unwrap()
                    .retain(|stored_id| stored_id != &id);

                if let Some(mut parent_node) = nodes.get_mut(&parent) {
                    parent_node.children.push(node.1);
                    parent_node.children.sort_by_key(|c| c.order_index);
                }
            } else {
                nodes.insert(node.1.id, node.1);
            }
        }
    }

    let mut root_nodes: Vec<ScrapeObject> = nodes
        .into_iter()
        .filter(|(_, node)| node.parent.is_none())
        .map(|(_, node)| node)
        .collect();

    root_nodes.sort_by_key(|c| c.order_index);

    root_nodes
}
