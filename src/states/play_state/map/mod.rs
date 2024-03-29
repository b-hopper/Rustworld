use super::{MAPSIZE_MAX_Y, MAPSIZE_MAX_X, Assets};
use super::entities::{ BuildableEntity, Buildable, Entities, Entity, Tile, Wall};
use ggez::GameResult;
use ggez::GameError::ResourceLoadError;
use noise::{ NoiseFn, Perlin };
use noise::Seedable;
use pathfinding::grid::Grid;
use pathfinding::prelude::{absdiff, astar};
use std::time::SystemTime;

mod mapgenerator;
use self::mapgenerator::MapGenerator;

const NOISESCALE: f64 = 0.05;

pub struct Map {
  pub tilemap: Vec<(Tile, u32)>,
  pub build_layer: Vec<Option<Box<Buildable>>>,
  pub costmap: Vec<usize>,

}

impl Map {
  pub fn new(assets: &Assets) -> Self {
    let seed = SystemTime::now();
    let elapsed = seed.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let mut map = MapGenerator::generate_map(elapsed.as_secs() as u32, assets);
      
    map
  }

  pub fn getpath(&mut self, from: Pos, to: Pos) -> GameResult<Vec<Pos>> {
    let result = astar(&from, |p| p.successors(&self.costmap), |p| p.distance(&to) / 3, |p| *p == to);
    match result {
      Some((result, weight)) => { Ok(result) },
      None => { Err(ggez::GameError::UnknownError("No path found".to_string())) },
    }
  }

  pub fn get_tile_at(&self, x: i32, y: i32) -> GameResult<&(Tile, u32)> {
    if !Map::check_bounds(x, y) {
      return Err(ResourceLoadError("Tile out of bounds".to_string()));
    }
    Ok(self.tilemap.get(getmapvecidx(x, y)).unwrap())
  }

  pub fn set_tile_at(&mut self, x: i32, y: i32, t: Tile) -> GameResult<()> {
    if !Map::check_bounds(x, y) {
         return Err(ResourceLoadError("Tile out of bounds".to_string()));
    }
    let idx = getmapvecidx(x, y);
    self.tilemap.remove(idx);
    let a = &t.alt.clone();
    self.tilemap.insert(idx, (t, *a));
    Ok(())

  }

  pub fn get_building_at(&mut self, x: i32, y: i32) -> bool {
    if !Map::check_bounds(x, y) {
      return false;
    }
    match self.build_layer.get(getmapvecidx(x,y)).unwrap() {
      Some(a) => { true }
      None    => { false }
    }
  }

  pub fn set_building_at<T: BuildableEntity + Copy + 'static>(&mut self, x: i32, y: i32, w: T, entities: &mut Entities) -> GameResult<()> {
    if !Map::check_bounds(x, y) {
      return Err(ResourceLoadError("Tile out of bounds".to_string()));
    }
    let idx = getmapvecidx(x, y);
    self.costmap.remove(idx);
    self.costmap.insert(idx, w.getmovecost());
    let a = Box::new(w);
    entities.add_building(w);
    self.build_layer.remove(idx);
    self.build_layer.insert(idx, Some(a));
    Ok(())
  }

  pub fn clear_building_at(&mut self, x: i32, y: i32, entities: &mut Entities) -> GameResult<()> {
    if !Map::check_bounds(x, y) {
      return Err(ResourceLoadError("Tile out of bounds".to_string()));
    }
    let idx = getmapvecidx(x, y);
    let w = self.build_layer.remove(idx);
    self.build_layer.insert(idx, None);
    self.costmap.remove(idx);

    let b = w.unwrap().getentityid();

    entities.remove_building(b); 
    
      
    let (i, j) = self.get_tile_at(x,y).unwrap();
    self.costmap.insert(idx, i.getmovecost()); 
    Ok(())
  }

  fn check_bounds(x: i32, y: i32) -> bool {
    if x < 0 || x >= MAPSIZE_MAX_X ||
       y < 0 || y >= MAPSIZE_MAX_Y {
         return false;
    }
    true
  }
}

pub fn getmapvecidx(x: i32, y: i32) -> usize { (x + (MAPSIZE_MAX_X * y)) as usize }

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pos(pub i32, pub i32);

impl Pos { // Pathfinding is expensive :(
  fn distance(&self, other: &Pos) -> usize {
    (absdiff(self.0, other.0) + absdiff(self.1, other.1)) as usize
  }
  fn successors(&self, costmap: &Vec<usize>) -> Vec<(Pos, usize)> {
    let &Pos(x, y) = self;
    let mut cost: usize = usize::min_value();
    let mut ret: Vec<(Pos, usize)> = Vec::new();
    if x > 0 {
      cost = *costmap.get((x-1 + ((y) * MAPSIZE_MAX_Y)) as usize).unwrap();
      if cost < usize::max_value() { ret.push((Pos(x-1, y),  cost)); }
    }
    if x < (MAPSIZE_MAX_X - 1) {
      cost = *costmap.get((x+1 + ((y) * MAPSIZE_MAX_Y)) as usize).unwrap();
      if cost < usize::max_value() { ret.push((Pos(x+1, y),  cost)); }
    }
    if y > 0 {
      cost = *costmap.get((x + ((y-1) * MAPSIZE_MAX_Y)) as usize).unwrap();
      if cost < usize::max_value() { ret.push((Pos(x, y-1), cost)); }
    }
    if y < (MAPSIZE_MAX_Y - 1) {
      cost = *costmap.get((x + ((y+1) * MAPSIZE_MAX_Y)) as usize).unwrap();
      if cost < usize::max_value() { ret.push((Pos(x, y+1),  cost)); }
    }
    ret
  }
  
}
