#![allow(dead_code)]

use std::collections::HashMap;

type Position = (usize, usize);

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq)]
enum Phenotype {
  Empty,
  Sand,
  Ceramic,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Neighbour {
  TopLeft = 0,
  TopMid,
  TopRight,
  MidLeft,
  MidRight,
  BotLeft,
  BotMid,
  BotRight
}

impl std::convert::From<usize> for Neighbour {
  fn from(ne: usize) -> Neighbour {
    // only using because Neighbour has a usize representation
    unsafe { std::mem::transmute(ne as usize) }
  }
}

#[derive(Debug, Clone)]
struct Node {
  future_pheno: Option<Phenotype>,
  pheno: Phenotype,
  position: Position,
  is_seed: bool
}

impl Node {
  fn new(pheno: Phenotype, position: Position) -> Self {
    Node { future_pheno: None, pheno, position, is_seed: false }
  }

  fn update(&self, neighbours: HashMap<Neighbour, Option<Node>>, grid: &mut Grid) {
    match self.pheno {
      Phenotype::Empty => update_empty(self, neighbours, grid),
      Phenotype::Sand  => update_sand(self, neighbours, grid),
      Phenotype::Ceramic => update_ceramic(self, neighbours, grid)
    }
  }
  
  fn draw(&self) {
    print!("{}", phenotype_as_glyph(&self.pheno));
  }
}

#[derive(Debug)]
struct Grid {
  width: usize,
  height: usize,
  map: Vec<Vec<Node>>
}

impl Grid {
  fn new(width: usize, height: usize) -> Self {
    let mut map: Vec<Vec<Node>> = Vec::new();
    
    for i in 0..height {
      map.push(Vec::new());
      for j in 0..width {
        map[i].push(Node::new(Phenotype::Empty, (i, j)));
      }
    }
    
    Grid { width, height, map }
  }

  fn load(file_path: &'static str) -> Self {
    let file_bytes: Vec<u8> = std::fs::read(file_path)
      .expect("Failed to read file!");
    let file_str: String = String::from_utf8(file_bytes)
      .expect("UTF8 read failed!");
    let file_info = file_str
      .as_str()
      .split(&['\n', '\r'][..])
      .filter(|ele| { *ele != "" })
      .collect::<Vec<&str>>();

    let width = file_info[0]
      .parse::<usize>()
      .unwrap();
    let height = file_info[1]
      .parse::<usize>()
      .unwrap();
    
    let mut map: Vec<Vec<Node>> = Vec::new();

    for (i, row) in file_info
      .iter()
      .skip(2)
      .enumerate() {
      map.push(Vec::new());
      for (j, ele) in (*row)
        .as_bytes()
        .to_vec()
        .iter()
        .enumerate() {
        map[i].push(Node::new(glyph_as_phenotype(*ele as char), (i, j)));
      }
    }

    Grid { width, height, map }
  }

  fn save(&self, file_path: &'static str) {
    use std::io::Write;

    let mut file = std::fs::File::create(file_path)
      .expect("Create failed!");

    write!(file, "{}\n", self.width.to_string())
      .expect("Write failed!");

    write!(file, "{}\n", self.height.to_string())
      .expect("Write failed!");

    for i in 0..self.height {
      for j in 0..self.width {
        write!(file, "{}", phenotype_as_glyph(&self.get_ref_node((i, j)).unwrap().pheno))
          .expect("Write failed!");
      }
      write!(file, "\n")
        .expect("Write failed!");
    }
  }

  fn set_node(&mut self, position: Position, new_node: Node) -> bool {
    if position.0 >= self.height
    || position.1 >= self.width {
      return false;
    }
    
    self.map[position.0][position.1] = new_node;
    
    true
  }

  fn set_seed(&mut self, position: Position, new_seed: Node) -> bool {
    if position.0 >= self.height
    || position.1 >= self.width {
      return false;
    }
    
    self.map[position.0][position.1] = new_seed;
    self.map[position.0][position.1].is_seed = true;
    
    true
  }

  fn get_node(&self, position: Position) -> Option<Node> {
    if position.0 >= self.height
    || position.1 >= self.width {
      return None;
    }
    
    Some(self.map[position.0][position.1].clone())
  }

  fn get_ref_node(&self, position: Position) -> Option<&Node> {
    if position.0 >= self.height
    || position.1 >= self.width {
      return None;
    }
    
    Some(&self.map[position.0][position.1])
  }

  fn get_mut_ref_node(&mut self, position: Position) -> Option<&mut Node> {
    if position.0 >= self.height
    || position.1 >= self.width {
      return None;
    }
    
    Some(&mut self.map[position.0][position.1])
  }

  fn draw(&self) {
    for i in 0..self.height {
      for j in 0..self.width {
        self.get_ref_node((i, j)).unwrap().draw(); // should never fail
      }
      print!("\n");
    }
  }

  fn update(&mut self) {
    for i in 0..self.height {
      for j in 0..self.width {
        // get rid of hashmap and add a tuple field for Option Phenotype to the Neighbour enum
        // I would really like this to be Option<&Node> for efficiency, but rust's borrow checker 
        // is a pain in the ass so im working around the borrow issues with Clone
        let mut neighbours: HashMap<Neighbour, Option<Node>> = HashMap::new();

        neighbours.insert(Neighbour::TopLeft,  self.get_node((i.overflowing_sub(1).0, j.overflowing_sub(1).0)));
        neighbours.insert(Neighbour::TopMid,   self.get_node((i.overflowing_sub(1).0, j)));
        neighbours.insert(Neighbour::TopRight, self.get_node((i.overflowing_sub(1).0, j.overflowing_add(1).0)));
        neighbours.insert(Neighbour::MidLeft,  self.get_node((i, j.overflowing_sub(1).0)));
        neighbours.insert(Neighbour::MidRight, self.get_node((i, j.overflowing_add(1).0)));
        neighbours.insert(Neighbour::BotLeft,  self.get_node((i.overflowing_add(1).0, j.overflowing_sub(1).0)));
        neighbours.insert(Neighbour::BotMid,   self.get_node((i.overflowing_add(1).0, j)));
        neighbours.insert(Neighbour::BotRight, self.get_node((i.overflowing_add(1).0, j.overflowing_add(1).0)));
        
        if let Some(curr_node) = self.get_node((i, j)) {
          curr_node.update(neighbours, self);
        }
      }
    }

    // perform update
    self.map
      .iter_mut()
      .flatten()
      .for_each(|ele| {
        if let Some(p) = ele.future_pheno.clone() {
          if !ele.is_seed {
            ele.pheno = p;
            ele.future_pheno = None;
          }
        }
      });
  }

  fn run<F: FnOnce(&mut Grid) -> ()>(&mut self, initialization_fn: F) {
    initialization_fn(self);
    let mut iteration = 0;
    loop {
      clear_console();
      self.draw();
      self.update();
      println!("Iteration: {}", iteration);
      iteration += 1;
      std::thread::sleep(std::time::Duration::from_millis(150));
    }
  }
}

fn phenotype_as_glyph(pheno: &Phenotype) -> char {
  match *pheno {
    Phenotype::Empty   => ' ',
    Phenotype::Sand    => '+',
    Phenotype::Ceramic => '#',
    _ => panic!("Unimplemented phenotype!")
  }
}

fn glyph_as_phenotype(glyph: char) -> Phenotype {
  match glyph {
    ' ' => Phenotype::Empty,
    '+' => Phenotype::Sand,
    '#' => Phenotype::Ceramic,
    _ => panic!("Unimplemented glyph!")
  }
}

fn update_empty(_node: &Node, _neighbours: HashMap<Neighbour, Option<Node>>, _grid: &mut Grid) {
  ()
}

fn update_ceramic(_node: &Node, _neighbours: HashMap<Neighbour, Option<Node>>, _grid: &mut Grid) {
  () // stay constant
}

fn update_sand(node: &Node, neighbours: HashMap<Neighbour, Option<Node>>, grid: &mut Grid) {

  for target in [Neighbour::BotMid,
                 Neighbour::BotRight,
                 Neighbour::BotLeft].iter() {
    if let Some(neighbour) = neighbours.get(target).unwrap() {
      if neighbour.pheno == Phenotype::Empty {
        grid.get_mut_ref_node(node.position).unwrap().future_pheno = Some(Phenotype::Empty);
        grid.get_mut_ref_node(neighbour.position).unwrap().future_pheno = Some(Phenotype::Sand);
        return;
      }
    }
  }
  
  grid.get_mut_ref_node(node.position).unwrap().future_pheno = Some(node.pheno.clone()); // stay constant
}

fn clear_console() {
  print!("\x1B[2J\x1B[1;1H\n");
}

fn main() {
  let mut grid = Grid::load("resources/HappyFace.ssim");
  grid.run(|_| {});

  /*
  let mut grid = Grid::new(20, 30);
  
  grid.run(move |g| {
    g.set_seed((0, 4), Node::new(Phenotype::Sand, (0, 4)));
    g.set_seed((1, 14), Node::new(Phenotype::Sand, (1, 14)));

    g.set_node((5, 4), Node::new(Phenotype::Ceramic, (5, 4)));
    g.set_node((5, 5), Node::new(Phenotype::Ceramic, (5, 5)));
    g.set_node((5, 6), Node::new(Phenotype::Ceramic, (5, 6)));
    g.set_node((5, 7), Node::new(Phenotype::Ceramic, (5, 7)));
    g.set_node((5, 8), Node::new(Phenotype::Ceramic, (5, 8)));

    g.set_node((10, 2), Node::new(Phenotype::Ceramic, (10, 2)));
    g.set_node((10, 3), Node::new(Phenotype::Ceramic, (10, 3)));
    g.set_node((10, 4), Node::new(Phenotype::Ceramic, (10, 4)));
    g.set_node((10, 5), Node::new(Phenotype::Ceramic, (10, 5)));
    g.set_node((10, 6), Node::new(Phenotype::Ceramic, (10, 6)));
  });
  */
}
