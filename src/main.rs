use battery::units::ratio::percent;
use notify_rust::Notification;
use std::thread::sleep;
use std::time::Duration;

#[derive(PartialEq, Clone)]
enum Discharging {
    Normal,
    Low,
    Critical,
}

#[derive(PartialEq, Clone)]
enum Charging {
    Normal,
    High,
}

#[derive(PartialEq, Clone)]
enum State {
    Others,
    Discharging(Discharging),
    Charging(Charging),
}

const HIGH_THRESHOLD: f32 = 80.0;
const LOW_THRESHOLD: f32 = 30.0;
const CRITICAL_THRESHOLD: f32 = 10.0;
const UPDATE_INTERVAL: Duration = Duration::from_secs(30);

fn main() -> Result<(), battery::Error> {
    let manager = battery::Manager::new()?;

    let mut batteries: Vec<battery::Battery> =
        manager.batteries()?.filter_map(|x| x.ok()).collect();
    let n = batteries.len();
    let mut states = vec![State::Others; n];

    let mut noti = Notification::new();
    let noti = noti
        .summary("Battery Notifications")
        .urgency(notify_rust::Urgency::Critical);

    loop {
        for i in 0..n {
            let bat = batteries.get_mut(i).unwrap();
            let state = states.get_mut(i).unwrap();
            manager.refresh(bat)?;
            let p = bat.state_of_charge().get::<percent>();
            match bat.state() {
                battery::State::Discharging => {
                    if let State::Charging(_) | State::Others = state {
                        noti.body("Battery is discharging.").show().unwrap();
                    }

                    if p <= CRITICAL_THRESHOLD {
                        if *state != State::Discharging(Discharging::Critical) {
                            *state = State::Discharging(Discharging::Critical);
                            noti.body("Battery is critically low.").show().unwrap();
                        }
                    } else if p <= LOW_THRESHOLD {
                        if *state != State::Discharging(Discharging::Low) {
                            *state = State::Discharging(Discharging::Low);
                            noti.body("Battery is too low.").show().unwrap();
                        }
                    } else {
                        *state = State::Discharging(Discharging::Normal);
                    }
                }
                battery::State::Charging => {
                    if let State::Discharging(_) | State::Others = state {
                        noti.body("Battery is charging.").show().unwrap();
                    }

                    if p >= HIGH_THRESHOLD {
                        if *state != State::Charging(Charging::High) {
                            *state = State::Charging(Charging::High);
                            noti.body("Battery is too full.").show().unwrap();
                        }
                    } else {
                        *state = State::Charging(Charging::Normal);
                    }
                }
                _ => {
                    *state = State::Others;
                }
            }
        }

        sleep(UPDATE_INTERVAL);
    }
}
