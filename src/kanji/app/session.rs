use crate::kanji::domain::subject::{ReviewItem, Subject};
use std::io::{self, Write};

pub struct Session {
    pub items: Vec<ReviewItem>,
    pub current_index: usize,
}

impl Session {
    pub fn new(items: Vec<ReviewItem>) -> Self {
        Self { items, current_index: 0 }
    }

    pub fn next_question(&mut self) -> Option<&Subject> {
        if self.current_index >= self.items.len() {
            None
        } else {
            Some(&self.items[self.current_index].subject)
        }
    }

    pub fn submit_answer(&mut self, correct: bool) {
        if let Some(item) = self.items.get_mut(self.current_index) {
            item.seen += 1;
            if correct {
                item.correct += 1;
            }
        }
        self.current_index += 1;
    }

    pub fn run_loop(&mut self) {
        let mut score = 0usize;
        while let Some(subject) = self.next_question() {
            println!("{}", subject.characters);
            let prompt = format!("Meaning -> ");
            print!("{}", prompt);
            let _ = io::stdout().flush();
            let mut s = String::new();
            if io::stdin().read_line(&mut s).is_err() {
                println!("Input error, exiting");
                break;
            }
            let ans = s.trim().to_lowercase();
            let correct = subject
                .meanings
                .get(0)
                .map(|m| m.to_lowercase() == ans)
                .unwrap_or(false);
            if correct {
                println!("Correct!");
                score += 1;
            } else {
                println!("Wrong — answers: {:?}", subject.meanings);
            }
            self.submit_answer(correct);
        }
        println!("Finished — score: {}/{}", score, self.items.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_subject(ch: &str, meaning: &str) -> Subject {
        Subject {
            id: 1,
            characters: ch.to_string(),
            meanings: vec![meaning.to_string()],
            readings: vec![],
        }
    }

    #[test]
    fn session_progresses_and_scores() {
        let s1 = make_subject("一", "one");
        let s2 = make_subject("二", "two");
        let items = vec![ReviewItem::new(s1), ReviewItem::new(s2)];
        let mut session = Session::new(items);

        // initial
        assert_eq!(session.current_index, 0);
        assert!(session.next_question().is_some());

        // submit correct
        session.submit_answer(true);
        assert_eq!(session.current_index, 1);

        // submit incorrect
        session.submit_answer(false);
        assert_eq!(session.current_index, 2);

        assert!(session.next_question().is_none());
    }
}
