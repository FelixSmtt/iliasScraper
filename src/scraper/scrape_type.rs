use colored::Color;
use url::Url;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ScrapeType {
    Folder,
    Forum,
    MediaLibrary,
    Link,
    LinkLibrary,
    Submissions,
    Submission,
    File,
    Video,
    Calender,
    Ignore,
}

impl ScrapeType {
    pub fn get_color(&self) -> Color {
        match self {
            ScrapeType::Folder => Color::Blue,
            ScrapeType::Forum => Color::Yellow,
            ScrapeType::MediaLibrary => Color::Magenta,
            ScrapeType::Link => Color::Green,
            ScrapeType::LinkLibrary => Color::Blue,
            ScrapeType::Submissions => Color::Cyan,
            ScrapeType::Submission => Color::Cyan,
            ScrapeType::File => Color::Green,
            ScrapeType::Video => Color::Red,
            ScrapeType::Calender => Color::Green,
            ScrapeType::Ignore => Color::Green,
        }
    }

    fn handle_goto(url_string: &str, goto_index: usize) -> ScrapeType {
        let goto_index = goto_index + 10;
        let after_index = url_string[goto_index..].to_string();
        let goto = after_index.split('/').collect::<Vec<&str>>()[0];

        match goto {
            "fold" => ScrapeType::Folder,
            "frm" => ScrapeType::Forum,
            "exc" => ScrapeType::Submissions,
            "book" => ScrapeType::Calender,
            "svy" => ScrapeType::Ignore, // Survey Block
            _ => {
                println!("{}, goto not handled for {}", goto, url_string);
                ScrapeType::Ignore
            }
        }
    }

    fn handle_cmd(cmd: &str) -> ScrapeType {
        match cmd {
            "forward" => ScrapeType::MediaLibrary,
            "sendfile" => ScrapeType::File,
            "downloadFile" => ScrapeType::File, // In Submission Uploads,
            "download" => ScrapeType::File,     // From direct download videos in MediaLibrary
            "calldirectlink" => ScrapeType::Link,
            "callLink" => ScrapeType::Link,
            "streamVideo" => ScrapeType::Video,
            _ => ScrapeType::LinkLibrary,
        }
    }

    pub fn from_url(url: &Url) -> ScrapeType {
        let url_string = url.as_str();
        let goto_index_opt = url_string.find("/goto.php/");
        if let Some(goto_index) = goto_index_opt {
            return Self::handle_goto(url_string, goto_index);
        }

        let cmd_pair = url.query_pairs().find(|(key, _)| key == "cmd");
        if let Some((_, cmd)) = cmd_pair {
            return Self::handle_cmd(cmd.as_ref());
        }

        let ass_id_pair = url.query_pairs().find(|(key, _)| key == "ass_id");
        if ass_id_pair.is_some() {
            return ScrapeType::Submission;
        }

        ScrapeType::Ignore
    }
}
