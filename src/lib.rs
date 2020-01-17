use std::cmp;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const MOVE_NORMAL_COST: i32 = 10;
const MOVE_DIAGONAL_COST: i32 = 14;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum NodeStates {
  DEFAULT = 0,
  WALL = 1,
  START = 2,
  END = 3,
  PATH = 4,
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Node {
  index: usize,
  parent_index: Option<usize>,
  parent_diagonal: bool,
  state: NodeStates,
  x: i32,
  y: i32,
  g: i32,
  h: i32,
  f: i32,
}

impl Node {
  pub fn new(index: usize, x: i32, y: i32) -> Node {
    Node {
      index: index,
      parent_index: None,
      parent_diagonal: false,
      state: NodeStates::DEFAULT,
      x: x,
      y: y,
      g: 0,
      h: 0,
      f: 0,
    }
  }

  pub fn set_h(&mut self, goal: Node) {
    let dx: i32 = (self.x - goal.x).abs();
    let dy: i32 = (self.y - goal.y).abs();

    self.h =
      MOVE_NORMAL_COST * (dx + dy) + (MOVE_DIAGONAL_COST - 2 * MOVE_NORMAL_COST) * cmp::min(dx, dy);
  }

  pub fn set_g_f(&mut self, parent: &Node, diagonal: bool) {
    self.g = parent.g
      + if diagonal {
        MOVE_DIAGONAL_COST
      } else {
        MOVE_NORMAL_COST
      };

    self.f = self.g + self.h;
  }
}

impl PartialEq for Node {
  fn eq(&self, other: &Self) -> bool {
    self.index == other.index
  }
}

#[wasm_bindgen]
#[allow(dead_code)]
pub struct Grid {
  width: i32,
  height: i32,
  start_index: usize,
  end_index: usize,
  path: Vec<u16>,
  nodes: Vec<Node>,
}

#[wasm_bindgen]
#[allow(dead_code)]
impl Grid {
  pub fn new(width: i32, height: i32) -> Grid {
    let mut nodes = Vec::new();
    let mut index = 0;

    for col in 0..height {
      for row in 0..width {
        nodes.push(Node::new(index, row as i32, col as i32));
        index += 1;
      }
    }

    Grid {
      width: width,
      height: height,
      start_index: 0,
      end_index: (width * height - 1) as usize,
      path: Vec::new(),
      nodes: nodes,
    }
  }

  fn get_neighbours(&self, node: Node) -> Vec<Node> {
    let mut neighbours: Vec<Node> = Vec::new();
    let mut pos: u8 = 0;

    for y in (node.y - 1)..=(node.y + 1) {
      for x in (node.x - 1)..=(node.x + 1) {
        if x == node.x && y == node.y {
          pos += 1;
          continue;
        }

        if x >= 0 && x < self.width && y >= 0 && y < self.height {
          let mut neighbour: Node = self.nodes[(y * self.width + x) as usize];

          if neighbour.state != NodeStates::WALL {
            if pos % 2 == 0 {
              neighbour.parent_diagonal = true;
            }

            neighbours.push(neighbour);
          }
        }

        pos += 1;
      }
    }

    neighbours
  }

  fn get_index(&self, x: i32, y: i32) -> usize {
    (x * self.width + y) as usize
  }

  pub fn nodes(&self) -> *const NodeStates {
    let mut nodes: Vec<NodeStates> = Vec::new();

    for node in &self.nodes {
      nodes.push(node.state);
    }
    nodes.as_ptr()
  }

  pub fn set_start(&mut self, x: i32, y: i32) {
    self.nodes[self.start_index].state = NodeStates::DEFAULT;
    self.start_index = self.get_index(x, y);
    self.nodes[self.start_index].state = NodeStates::START;
  }

  pub fn set_end(&mut self, x: i32, y: i32) {
    self.nodes[self.end_index].state = NodeStates::DEFAULT;
    self.end_index = self.get_index(x, y);
    self.nodes[self.end_index].state = NodeStates::END;
  }

  pub fn add_wall(&mut self, x: i32, y: i32) {
    let index: usize = self.get_index(x, y);
    let node: &mut Node = &mut self.nodes[index];

    if node.state == NodeStates::DEFAULT || node.state == NodeStates::PATH {
      node.state = NodeStates::WALL
    }
  }

  pub fn remove_wall(&mut self, x: i32, y: i32) {
    let index: usize = self.get_index(x, y);
    let node: &mut Node = &mut self.nodes[index];

    if node.state == NodeStates::WALL {
      node.state = NodeStates::DEFAULT
    }
  }

  pub fn get_path_count(&self) -> usize {
    self.path.len()
  }

  pub fn get_path(&self) -> *const u16 {
    self.path.as_ptr()
  }

  pub fn clear_path(&mut self) {
    for node in self
      .nodes
      .iter_mut()
      .filter(|node| node.state == NodeStates::PATH)
    {
      node.state = NodeStates::DEFAULT;
    }
  }

  pub fn a_star(&mut self) {
    self.path = Vec::new();

    let mut start: Node = self.nodes[self.start_index];
    let end: Node = self.nodes[self.end_index];
    let mut open_nodes: Vec<Node> = vec![start];
    let mut closed_nodes: Vec<Node> = Vec::new();
    let mut current_node: Node;

    start.set_h(self.nodes[self.end_index]);
    start.f = start.g + start.h;

    while open_nodes.len() > 0 {
      open_nodes.sort_by(|x, y| x.f.cmp(&y.f));

      current_node = open_nodes.swap_remove(0);
      closed_nodes.push(current_node);

      if current_node.index == end.index {
        let mut path: Vec<u16> = vec![current_node.index as u16];

        while current_node.parent_index != None {
          match closed_nodes
            .iter()
            .find(|&n| n.index == current_node.parent_index.unwrap())
          {
            Some(parent) => current_node = *parent,
            None => break,
          }

          if current_node.state != NodeStates::START {
            self.nodes[current_node.index].state = NodeStates::PATH;
          }

          path.push(current_node.index as u16);
        }

        path.reverse();
        self.path = path;
      }

      for neighbour in self.get_neighbours(current_node).iter_mut() {
        neighbour.parent_index = Some(current_node.index);

        if !closed_nodes.contains(neighbour) {
          neighbour.set_h(end);
          neighbour.set_g_f(&current_node, neighbour.parent_diagonal);

          match open_nodes.iter_mut().find(|n| n.index == neighbour.index) {
            Some(node) => {
              if neighbour.g < node.g {
                node.g = neighbour.g;
                node.parent_index = neighbour.parent_index;
              }
            }
            None => open_nodes.push(*neighbour),
          }
        }
      }
    }
  }
}
