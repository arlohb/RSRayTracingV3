use std::collections::VecDeque;
use crate::Time;

#[derive(Clone)]
pub struct Frame {
  pub time: f64,
  pub value: f64,
}

impl Frame {
  pub fn new(time: f64, value: f64) -> Self {
    Self {
      time,
      value,
    }
  }

  pub fn age(&self) -> f64 {
    Time::now_millis() - self.time
  }
}

pub struct History {
  max_age: f64,
  values: VecDeque<Frame>,
}

impl History {
  pub fn new(max_age: f64) -> History {
    History {
      max_age,
      values: VecDeque::new(),
    }
  }

  fn flush(&mut self) {
    self.values.clone().iter().filter(|frame| {
      frame.age() > self.max_age
    }).for_each(|_| {
      self.values.pop_front();
    });
  }

  pub fn add(&mut self, value: f64) {
    let now = Time::now_millis();
    self.values.push_back(Frame::new(now, value));

    self.flush();
  }

  pub fn values(&self, max_age: Option<f64>) -> Vec<Frame> {
    let max_age = max_age.unwrap_or(self.max_age);
    self.values
      .iter()
      .filter(|frame| {
        frame.age() < max_age
      })
      .cloned()
      .collect()
  }

  pub fn sum(&self, max_age: Option<f64>) -> f64 {
    self.values(max_age)
      .iter()
      .map(|frame| frame.value)
      .sum()
  }

  pub fn average(&self, max_age: Option<f64>) -> f64 {
    self.sum(max_age) / self.values(max_age).len().max(1) as f64
  }
}
