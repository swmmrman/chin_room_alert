use reqwest;
use std::process;
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use rppal::pwm;


fn main() {
    let mut max_temp: f32 = 70.9; 
    let offset: f32 = 2.0; // Current sensor overreads by ~2 degrees.
    max_temp += offset;
    let running = Arc::new(Mutex::new(true));
    let running_t = Arc::clone(&running);
    let running_m = Arc::clone(&running);
    ctrlc::set_handler(move || {
        let mut guard = running.lock().unwrap();
        println!("\rExiting please wait.");
        *guard = false;
        drop(guard);
    }).expect("Failed to set ctrl-c");
    let over_temp = Arc::new(Mutex::new(false));
    let over_t = Arc::clone(&over_temp);
    let buzzer_thread = std::thread::spawn(move || { 
        let buzzer_open = pwm::Pwm::new(pwm::Channel::Pwm0);
        let buzzer = match buzzer_open {
            Ok(p) => p,
            Err(e) => {
                println!("PWM Error:{}", e);
                process::exit(1)
            },
        };
        buzzer.set_frequency(4000.2f64, 0.5f64).unwrap();
        buzzer.set_polarity(pwm::Polarity::Normal).unwrap();
        loop {
            let ot = over_t.lock().unwrap();
            if *ot {
                drop(ot);
                match buzzer.enable() {
                Ok(_) => (),
                Err(_) => (),
                }
                sleep(Duration::from_millis(50));
                match buzzer.disable() {
                    Ok(_) => (),
                    Err(_) => (),
                }
                sleep(Duration::from_millis(100));
            }
            else {
                sleep(Duration::from_secs(1));
            }
            let guard = running_t.lock().unwrap();
            if !*guard {
                break;
            }
            drop(guard);
        }
    });
    loop {
        let ans = match reqwest::blocking::get("http://192.168.1.134/temp_in.txt") {
            Ok(r) =>  match r.text() {
                Ok(t) => t,
                Err(_) => "8888".to_owned(),
            }
            Err(e) => { 
                eprintln!("{:?}", e);
                "404".to_owned()
            }
        };
        let mut set_ot = over_temp.lock().unwrap();
        if &ans.parse::<f32>().unwrap() > &max_temp {
            *set_ot = true;
        }
        else {
            *set_ot = false;
        }
        drop(set_ot);
        let mut outfile = std::fs::OpenOptions::new()
            .write(true)
            .create(false)
            .open("/tmp/ledpipe")
            .unwrap();
        match outfile.write(format!("{}\n",ans).as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                println!("Pipe failure: {}", e);
                process::exit(1)
            }
        }
        match outfile.flush() {
            Ok(_) => (),
            Err(e) => {
                println!("Flush failure: {}, Call the plumber.", e)
            }
        }
        drop(outfile);
        let gaurd = running_m.lock().unwrap();
        if !*gaurd {
            break;
        }
        drop(gaurd);
        sleep(Duration::from_millis(2000));
    }
    buzzer_thread.join().unwrap();
}
