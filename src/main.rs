use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::thread;

const MAX_DEPTH: u32 = 100;

/// Reads filename from command line arguments, reads the file and detects a pattern for each line.
fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    println!("Usage: wunderpahkina_vol9 [filename]");
    return;
  }

  let filename = &args[1];

  let mut file = File::open(filename).expect("File not found.");

  let mut content = String::new();

  file
    .read_to_string(&mut content)
    .expect("Could not read the file.");

  let mut children = vec![];
  let lines = content.lines().enumerate();

  for (index, line) in lines {
    let s = String::from(line);

    // Detect each pattern in a new thread.
    children.push(thread::spawn(move || {
      // Return a tuple with index + Pattern.
      (index, Row::detect_pattern(&s, MAX_DEPTH))
    }));
  }

  let mut results = vec![];

  // Join all child threads and push results to vector.
  for child in children {
    let result = child.join().unwrap();
    results.push(result);
  }

  // Sort results by line index.
  results.sort_by(|a, b| a.0.cmp(&b.0));

  for result in results {
    println!("{}", result.1);
  }
}

/// Enumeration for each possible pattern definition.
#[derive(Debug, PartialEq)]
enum Pattern {
  Blinking,
  Gliding,
  Vanishing,
  Other,
}

impl fmt::Display for Pattern {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Pattern::Blinking => "blinking",
        Pattern::Gliding => "gliding",
        Pattern::Vanishing => "vanishing",
        Pattern::Other => "other",
      }
    )
  }
}

/// Model for a line in a grid paper.
/// Filled squares are stored in a hash set.
/// Additionally stores minimum and maximum values.
struct Row {
  values: HashSet<i32>,
  min: i32,
  max: i32,
}

impl PartialEq for Row {
  fn eq(&self, other: &Row) -> bool {
    self.values.eq(&other.values)
  }
}

impl Row {
  /// Constructs a new Row with empty set of values.
  fn new() -> Row {
    Row {
      values: HashSet::new(),
      min: 0,
      max: 0,
    }
  }

  /// Constructs a new row from a string.
  ///
  /// # Examples
  ///
  /// ```
  /// // set has values (2, 3, 5, 6)
  /// let row = Row::from_string("..##.##");
  /// ```
  fn from_string(s: &str) -> Row {
    let mut row = Row::new();
    let mut index = 0;

    for c in s.chars() {
      if c == '#' {
        row.insert(index);
      }

      index += 1;
    }

    row
  }

  /// Constructs the next row according to wunderpahkina-vol9 rules.
  ///
  /// # Examples
  ///
  /// ```
  /// let row = Row::from_string(".#.##.");
  /// let next = row.next(); //  "..##.#."
  /// ```
  fn next(&self) -> Row {
    let mut row = Row::new();

    for num in (self.min - 1)..(self.max + 2) {
      let insert = match self.values.contains(&num) {
        true => self.test_rule_2(num),
        false => self.test_rule_1(num),
      };

      if insert {
        row.insert(num);
      }
    }

    row
  }

  /// Insert value to set and update min & max values.
  fn insert(&mut self, value: i32) {
    if self.values.is_empty() {
      self.min = value;
      self.max = value;
    } else if value < self.min {
      self.min = value;
    } else if value > self.max {
      self.max = value;
    }

    self.values.insert(value);
  }

  /// Counts the number of neighbors up to 2 indices away at the specified index.
  fn calc_neighbor_sum(&self, index: i32) -> u32 {
    [index - 2, index - 1, index + 1, index + 2]
      .iter()
      .map(|x| match self.values.contains(x) {
        true => 1,
        false => 0,
      }).sum()
  }

  /// Test if wunderpahkina-vol9 rule #1 applies.
  fn test_rule_1(&self, num: i32) -> bool {
    let sum = self.calc_neighbor_sum(num);

    sum == 2 || sum == 3
  }

  /// Test if wunderpahkina-vol9 rule #2 applies.
  fn test_rule_2(&self, num: i32) -> bool {
    let sum = self.calc_neighbor_sum(num);

    sum == 2 || sum == 4
  }

  /// Shift values in set by given offset and test if other row contains those values.
  ///
  /// # Examples
  ///
  /// ```
  /// let row1 = Row::from_string("##..");
  /// let row2 = Row::from_string("..##");
  /// row1.eq_shift(&row2, 2) // true
  /// ```
  fn eq_shift(&self, other: &Row, offset: i32) -> bool {
    self
      .values
      .iter()
      .map(|x| x + offset)
      .all(|x| other.values.contains(&x))
  }

  /// Test if the pattern is gliding
  fn is_gliding(&self, other: &Row) -> bool {
    self.values.len() == other.values.len()
      && self.min != other.min
      && self.eq_shift(other, other.min - self.min)
  }

  /// Detect the pattern for a row by calling a recursive function up to max_depth - 1 times.
  fn detect_pattern(s: &str, max_depth: u32) -> Pattern {
    let mut previous_rows = Vec::new();

    // Add the initial row to previous rows.
    previous_rows.push(Row::from_string(s));

    let row = Row::from_string(s);
    row.detect_pattern_recursive(max_depth, 2, &mut previous_rows)
  }

  fn detect_pattern_recursive(
    &self,
    max_depth: u32,
    current_depth: u32,
    previous_rows: &mut Vec<Row>,
  ) -> Pattern {
    let next = self.next();

    if next.values.is_empty() {
      return Pattern::Vanishing;
    }

    // The last row can't equal the next row.
    if previous_rows[..previous_rows.len() - 1]
      .iter()
      .any(|x| next.eq(x))
    {
      return Pattern::Blinking;
    }

    if previous_rows[..].iter().any(|x| next.is_gliding(x)) {
      return Pattern::Gliding;
    }

    if current_depth >= max_depth {
      return Pattern::Other;
    }

    previous_rows.push(self.next());
    return next.detect_pattern_recursive(max_depth, current_depth + 1, previous_rows);
  }
}

// Test suite for Row struct.
#[cfg(test)]
mod row_tests {
  use super::*;

  macro_rules! calc_neighbor_sum_tests {
    ($($name:ident: $data:expr)*) => {
      $(
        #[test]
        fn $name() {
          let (s, index, expected) = $data;
          let row = Row::from_string(&s);
          let sum = row.calc_neighbor_sum(index);
          assert_eq!(sum, expected);
        }
      )*
    }
  }

  calc_neighbor_sum_tests! {
    calc_neighbor_sum_01: ("#####", 2, 4)
    calc_neighbor_sum_02: (".####", 2, 3)
    calc_neighbor_sum_03: ("#.###", 2, 3)
    calc_neighbor_sum_04: ("##.##", 2, 4)
    calc_neighbor_sum_05: ("###.#", 2, 3)
    calc_neighbor_sum_06: ("####.", 2, 3)
    calc_neighbor_sum_07: ("..###", 2, 2)
    calc_neighbor_sum_08: (".#.##", 2, 3)
    calc_neighbor_sum_09: (".##.#", 2, 2)
    calc_neighbor_sum_10: (".###.", 2, 2)
    calc_neighbor_sum_11: ("...##", 2, 2)
    calc_neighbor_sum_12: ("..#.#", 2, 1)
    calc_neighbor_sum_13: ("..##.", 2, 1)
    calc_neighbor_sum_14: ("....#", 2, 1)
    calc_neighbor_sum_15: ("...#.", 2, 1)
    calc_neighbor_sum_16: (".....", 2, 0)

    calc_neighbor_sum_17: ("#####", 1, 3)
    calc_neighbor_sum_18: ("#####", 0, 2)
    calc_neighbor_sum_19: ("#####", -1, 2)
    calc_neighbor_sum_20: ("#####", -2, 1)
    calc_neighbor_sum_21: ("#####", -3, 0)
    calc_neighbor_sum_22: ("#####", 3, 3)
    calc_neighbor_sum_23: ("#####", 4, 2)
    calc_neighbor_sum_24: ("#####", 5, 2)
    calc_neighbor_sum_25: ("#####", 6, 1)
    calc_neighbor_sum_26: ("#####", 7, 0)
  }

  macro_rules! eq_shift_tests {
    ($($name:ident: $data:expr)*) => {
      $(
        #[test]
        fn $name() {
          let (s1, s2, offset, expected) = $data;
          let row1 = Row::from_string(&s1);
          let row2 = Row::from_string(&s2);
          let equals = row1.eq_shift(&row2, offset);
          assert_eq!(equals, expected);
        }
      )*
    }
  }

  eq_shift_tests! {
    eq_shift_test_01: ("####.", ".####", 1, true)
    eq_shift_test_02: ("####.", ".####", 0, false)
    eq_shift_test_03: ("####.", ".####", -1, false)

    eq_shift_test_04: ("####", "####", 0, true)
    eq_shift_test_05: ("####", "####", 1, false)
    eq_shift_test_06: ("####", "####", -1, false)

    eq_shift_test_07: ("...####", "####...", -3, true)
    eq_shift_test_08: ("##.##..", "..##.##", 2, true)
  }

  macro_rules! detect_pattern_tests {
    ($($name:ident: $data:expr)*) => {
      $(
        #[test]
        fn $name() {
          let (s, expected) = $data;
          // let row = Row::from_string(&s);
          let pattern = Row::detect_pattern(s, MAX_DEPTH);
          assert_eq!(pattern, expected);
        }
      )*
    }
  }

  detect_pattern_tests! {
    detect_pattern_test_01: ("##.######", Pattern::Gliding)
    detect_pattern_test_02: ("#.###......................#.###......................####......................###.#......................###.#", Pattern::Blinking)
    detect_pattern_test_03: ("#######", Pattern::Blinking)
    detect_pattern_test_04: ("#.#..#...####..##..##..##", Pattern::Blinking)
    detect_pattern_test_05: ("###.#....#.###", Pattern::Other)
    detect_pattern_test_06: ("########", Pattern::Vanishing)
    detect_pattern_test_07: ("##...#.###########", Pattern::Blinking)
    detect_pattern_test_08: ("#.#..#...####..##..##..##.....##", Pattern::Blinking)
    detect_pattern_test_09: ("#######.##.##.#.#....#.######", Pattern::Other)
    detect_pattern_test_10: ("#.######", Pattern::Gliding)
    detect_pattern_test_11: ("##....#.#....#.....#....#....#.....###.#", Pattern::Blinking)
    detect_pattern_test_12: ("#.###........................................................#######........................................................###.#", Pattern::Blinking)
    detect_pattern_test_13: ("#...###...#.#", Pattern::Blinking)
    detect_pattern_test_14: ("#...#.#..###...#", Pattern::Vanishing)
    detect_pattern_test_15: ("#########", Pattern::Blinking)
    detect_pattern_test_16: ("#######.##.##.#.#", Pattern::Gliding)
    detect_pattern_test_17: ("#...#...#...#...#...#...#...#...#...#...#", Pattern::Vanishing)
    detect_pattern_test_18: ("#..##.#..#", Pattern::Vanishing)
    detect_pattern_test_19: ("#.###...................................................###.#", Pattern::Blinking)
    detect_pattern_test_20: ("######", Pattern::Vanishing)
    detect_pattern_test_21: ("#...#...#...#...#...#...#...#...#...#...#....#######.##.##.#.#", Pattern::Gliding)
  }
}
