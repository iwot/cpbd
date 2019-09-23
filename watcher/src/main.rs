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
    URL(String),
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

fn main() -> Result<(), Error> {
    main_proc()
}

fn main_proc() -> Result<(), Error> {
    // クリップボード監視
    let (sender, receiver) = channel();

    // URL判定
    let re_url = Regex::new(r"^(https?|ftp)(://[-_.!~*'()a-zA-Z0-9;/?:@&=+$,%#]+)$").unwrap();

    let _handle1 = spawn(move || {
        let mut prev = String::new();
        loop {
            let s = get_clipboard_string();
            if let Ok(new_str) = s {
                if new_str != prev {
                    prev = new_str.clone();
                    if re_url.is_match(new_str.trim()) {
                        if sender.send(Message::URL(prev.clone())).is_err() {
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

    let memories = Arc::new(Mutex::new(vec![]));

    let memories_shower_data = Arc::clone(&memories);
    let memories_getter_data = Arc::clone(&memories);
    let _handle2 = spawn(move || {
        
        let quit_server = path!("exit").map(||{
            std::process::exit(0);
            ""
        });

        // クリップボード履歴をJSON形式で表示
        let memories_show = path!("memories").map(move || {
            let mut data = memories_shower_data.lock().unwrap().clone();
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
            let data:Vec<String> = memories_getter_data.lock().unwrap().clone();
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
        let routes = warp::get2().and(memories_show.or(memories_get).or(quit_server));
        warp::serve(routes).run(([127, 0, 0, 1], 3030));
    });

    // クリップボード履歴更新クロージャ
    let memory_updator = |new_test:String| {
        let mut memories = memories.lock().unwrap();
        memories.push(new_test);
        if memories.len() > MAX_MEMORIES {
            memories.remove(0);
        }
    };

    // レシーバーからの返却により分岐
    for result in receiver {
        match result {
            Message::URL(url) => {
                println!("URL");
                memory_updator(url);
            },
            Message::Text(txt) => {
                println!("TEXT");
                memory_updator(txt);
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
