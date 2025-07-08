use reqwest;
use std::process;
use std::{io::Write};
use std::thread::{self, sleep};
use std::time::Duration;
use rppal::{gpio,pwm};


fn main() {
    let max_temp = 50.0f32;//72.9f32;
    let mut over_temp = false;
    let _buzzer_thread = std::thread::spawn(move || { 
        let buzzer_open = pwm::Pwm::new(pwm::Channel::Pwm0);
        let buzzer = match buzzer_open {
            Ok(p) => p,
            Err(e) => {
                println!("PWM Error:{}", e);
                process::exit(1)
            },
        };
        let over_temp = true;
        buzzer.set_frequency(1000.2f64, 0.5f64).unwrap();
        buzzer.set_polarity(pwm::Polarity::Normal).unwrap();
        loop {
            if over_temp {
                for _ in 0..4 {
                    match buzzer.enable() {
                    Ok(_) => (),
                    Err(_) => (),
                    }
                    sleep(Duration::from_millis(10000));
                    match buzzer.disable() {
                        Ok(_) => (),
                        Err(_) => (),
                    }
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
        if &ans.parse::<f32>().unwrap() > &max_temp {
            over_temp = true;
        }
        else {
            over_temp = false;
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
