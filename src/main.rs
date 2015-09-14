extern crate rand;
extern crate num_cpus;

use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use rand::Rng;

struct InData {
    id: u32,
}

struct OutData {
    id: u32,
    thread_id: u32,
    text: String,
}

struct SharedImmutableData {
    value: String,
}

fn main() {
    // Creating an immutable threadsafe data
    let data = Arc::new(SharedImmutableData { value: "A bautiful data".to_string() });

    // creating channels for communication between main thread and childs
    let (tx_main, rx_main): (Sender<InData>, Receiver<InData>) = mpsc::channel();
    let main_receiver = Arc::new(Mutex::new(rx_main));
    let (tx_thread, rx_thread): (Sender<OutData>, Receiver<OutData>) = mpsc::channel();

    let nb_thread = num_cpus::get() as u32;
    let nb_tasks = 128;

    println!("{} cpus detected. Launch {} tasks on {} threads", nb_thread, nb_tasks, nb_thread);

    for id in 0..nb_thread {
        // set up datas for the thread 
        let rx = main_receiver.clone();
        let tx = tx_thread.clone();
        let common_data = data.clone();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                // receive data and break loop if an Error occur
                let data = rx.lock().unwrap().recv();
                if data.is_err() {
                    break;
                }

                // random sleep to simulate different tasks process
                thread::sleep_ms(rng.gen::<u32>() % 200);

                // send data to main thread
                let out = OutData { id: data.unwrap().id, thread_id: id, text: common_data.value.clone() };
                tx.send(out).unwrap();
            }
        });
    }

    // main thread create and send data (task) to child
    for id_task in 0..nb_tasks {
        let d = InData { id: id_task };
        tx_main.send(d).unwrap();
    }

    // results: waiting for each tasks
    for _ in 0..nb_tasks {
        let answer = rx_thread.recv().unwrap();
        println!("T: {} : {} n°{}", answer.thread_id, answer.text, answer.id);
    }
}
