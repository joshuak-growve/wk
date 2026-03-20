pub mod domain;
pub mod app;
pub mod infrastructure;

use crate::kanji::domain::subject::Subject;
use crate::kanji::domain::subject::ReviewItem;

pub fn run() {
    println!("Kanji practice — WaniKani client");
    match infrastructure::WaniKaniClient::new_from_env() {
        Ok(client) => {
            match client.get("/subjects?types=kanji") {
                Ok(v) => {
                    // v is a JSON object with `data` array
                    if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
                        let mut items: Vec<ReviewItem> = Vec::new();
                        for entry in arr.iter() {
                            if let Some(sub) = Subject::from_wanikani_value(entry) {
                                items.push(ReviewItem::new(sub));
                            }
                        }
                        if items.is_empty() {
                            println!("No kanji subjects found.");
                            return;
                        }
                        let mut session = app::session::Session::new(items);
                        session.run_loop();
                    } else {
                        println!("Unexpected response shape from API: missing data array");
                    }
                }
                Err(e) => {
                    println!("Error fetching subjects: {:?}", e);
                }
            }
        }
        Err(e) => println!("Failed to create WaniKani client: {:?}", e),
    }
}
