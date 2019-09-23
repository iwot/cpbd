// cargo run -p watcher
extern crate clipboard_win;
extern crate serde;
extern crate regex;

use clipboard_win::{set_clipboard_string, get_clipboard_string};
use std::thread;
use std::time::Duration;
use std::thread::spawn;
use std::sync::mpsc::{channel};
use std::io::Error;
use warp::{self, path, Filter};
use std::sync::{Mutex, Arc};
use serde::{Serialize, Deserialize};
use regex::Regex;

const MAX_MEMORIES: usize = 1000;

#[derive(Debug)]
enum Message {
    Text(String),
    URL(Vec<String>, String), // URLのリストと、元テキスト
}

#[derive(Serialize, Deserialize, Debug)]
struct ShowMemories {
    data: Vec<ShowMemory>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShowMemory {
    index: usize,
    value: String,
}

#[derive(Debug, Clone)]
struct Memories {
    text_list: Vec<String>,
    url_list: Vec<String>,
}

fn main() -> Result<(), Error> {
    main_proc()
}

fn main_proc() -> Result<(), Error> {
    // クリップボード監視
    let (sender, receiver) = channel();

    // URL抜き出し
    let re_urls = Regex::new(r"(https?|ftp)(://[-_.!~*'()a-zA-Z0-9;/?:@&=+$,%#]+)").unwrap();

    let get_urls = move |text:String| -> Vec<String> {
        let mut result = vec![];
        for caps in re_urls.captures_iter(text.as_str()) {
            result.push(caps[0].to_string());
        }
        result
    };

    let _handle1 = spawn(move || {
        let mut prev = String::new();
        loop {
            let s = get_clipboard_string();
            if let Ok(new_str) = s {
                if new_str != prev {
                    prev = new_str.clone();
                    let urls = get_urls(new_str);
                    if urls.len() > 0 {
                        if sender.send(Message::URL(urls, prev.clone())).is_err() {
                            println!("error in thread");
                            break;
                        }
                    } else {
                        if sender.send(Message::Text(prev.clone())).is_err() {
                            println!("error in thread");
                            break;
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(3));
        }
    });

    // クリップボード履歴管理のためのAPIサーバー

    // let memories = Arc::new(Mutex::new(vec![]));
    let memories = Arc::new(Mutex::new(Memories {text_list:vec![], url_list:vec![]}));

    let text_shower_data = Arc::clone(&memories);
    let text_getter_data = Arc::clone(&memories);
    let url_shower_data = Arc::clone(&memories);
    let url_getter_data = Arc::clone(&memories);
    let _handle2 = spawn(move || {
        
        let quit_server = path!("exit").map(||{
            std::process::exit(0);
            ""
        });

        // クリップボード履歴をJSON形式で表示
        let memories_show = path!("memories").map(move || {
            let mut data = text_shower_data.lock().unwrap().text_list.clone();
            data.reverse();

            let mut output_data = ShowMemories { data: vec![] };

            let mut count = 0;
            for s in data {
                count += 1;
                output_data.data.push(ShowMemory{index:count, value:s});
            }
            let serialized = serde_json::to_string_pretty(&output_data).unwrap();
            serialized
        });

        // インデックスで指定されたクリップボード履歴をクリップボードにコピー
        let memories_get = path!("memory" / usize).map(move |index:usize| {
            let data:Vec<String> = text_getter_data.lock().unwrap().text_list.clone();
            let max = data.len();
            let index = max as i32 - index as i32;
            if index >= 0 && index < max as i32 {
                let result = data[index as usize].clone();
                set_clipboard_string(result.clone().as_str()).expect("Set clipboard failure");
                result
            } else {
                "".to_string()
            }
        });

        // クリップボード履歴をJSON形式で表示
        let urls_show = path!("urls").map(move || {
            let mut data = url_shower_data.lock().unwrap().url_list.clone();
            data.reverse();

            let mut output_data = ShowMemories { data: vec![] };

            let mut count = 0;
            for s in data {
                count += 1;
                output_data.data.push(ShowMemory{index:count, value:s});
            }
            let serialized = serde_json::to_string_pretty(&output_data).unwrap();
            serialized
        });

        // インデックスで指定されたクリップボード履歴をクリップボードにコピー
        let url_get = path!("url" / usize).map(move |index:usize| {
            let data:Vec<String> = url_getter_data.lock().unwrap().url_list.clone();
            let max = data.len();
            let index = max as i32 - index as i32;
            if index >= 0 && index < max as i32 {
                let result = data[index as usize].clone();
                set_clipboard_string(result.clone().as_str()).expect("Set clipboard failure");
                result
            } else {
                "".to_string()
            }
        });

        let routes = warp::get2().and(
            memories_show.or(memories_get).or(quit_server).or(urls_show).or(url_get)
        );
        warp::serve(routes).run(([127, 0, 0, 1], 3030));
    });

    // クリップボード（テキスト）履歴更新クロージャ
    let memory_text_updator = |new_test:String| {
        let mut memories = memories.lock().unwrap();
        memories.text_list.push(new_test);
        if memories.text_list.len() > MAX_MEMORIES {
            memories.text_list.remove(0);
        }
    };

    // クリップボード（テキスト）履歴更新クロージャ
    let memory_url_updator = |urls:Vec<String>| {
        let mut memories = memories.lock().unwrap();
        memories.url_list.extend(urls);
        while memories.text_list.len() > MAX_MEMORIES {
            memories.text_list.remove(0);
        }
    };

    // レシーバーからの返却により分岐
    for result in receiver {
        match result {
            Message::URL(urls, txt) => {
                println!("URL");
                memory_text_updator(txt);
                memory_url_updator(urls);
            },
            Message::Text(txt) => {
                println!("TEXT");
                memory_text_updator(txt);
            },
        }
    }
    Ok(())
}

// fn read_clipboard() {
//     let clip_str = get_clipboard_string();
//     let cb = Clipboard::new().unwrap();

//     let clip_bmp = cb.get_bit_map();
//     let clip_file_list = cb.get_file_list();

//     if clip_str.is_ok() {
//         println!("STRING: {}", clip_str.unwrap());
//     } else if clip_bmp.is_ok() {
//         println!("BMP SIZE: {:?}", clip_bmp.unwrap().size());
//     } else if clip_file_list.is_ok() {
//         println!("FILE LIST LEN: {:?}", clip_file_list.unwrap().len());
//     } else {
//         println!("other");
//     }
// }
