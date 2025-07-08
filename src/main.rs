use reqwest;
use std::{process, sync::{Arc, Mutex}};
use std::io::Write;
use std::thread::{self, sleep};
use std::time::Duration;
use rppal::{gpio,pwm};


fn main() {
    let max_temp = 72.9f32;
    let over_temp = Arc::new(Mutex::new(false));
    let over_t = Arc::clone(&over_temp);
    let _buzzer_thread = std::thread::spawn(move || { 
        let buzzer_open = pwm::Pwm::new(pwm::Channel::Pwm0);
        let buzzer = match buzzer_open {
            Ok(p) => p,
            Err(e) => {
                println!("PWM Error:{}", e);
                process::exit(1)
            },
        };
        buzzer.set_frequency(1000.2f64, 0.5f64).unwrap();
        buzzer.set_polarity(pwm::Polarity::Normal).unwrap();
        loop {
            let ot = over_t.lock().unwrap();
            if *ot {
                for _ in 0..4 {
                    match buzzer.enable() {
                    Ok(_) => (),
                    Err(_) => (),
                    }
                    sleep(Duration::from_millis(100));
                    match buzzer.disable() {
                        Ok(_) => (),
                        Err(_) => (),
                    }
                    sleep(Duration::from_millis(900))
                }
            }
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
        sleep(Duration::from_millis(2000));
    }
}
