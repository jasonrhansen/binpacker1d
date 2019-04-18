use super::population::Population;
use super::*;

#[derive(Default)]
pub struct Optimizer {
    repos_lengths: Vec<usize>,
    demand_pieces: Vec<DemandPiece>,
    spacing: usize,
    random_seed: u64,
    multiplier: f64,
}

impl Optimizer {
    pub fn new(decimal_places: usize) -> Self {
        Optimizer {
            multiplier: 10.0f64.powf(decimal_places as f64),
            ..Default::default()
        }
    }

    pub fn add_stock_length(&mut self, repos_length: f64) -> &mut Self {
        self.repos_lengths.push(self.length_to_usize(repos_length));
        self
    }

    pub fn add_stock_lengths(&mut self, repos_lengths: &[f64]) -> &mut Self {
        for &repos_length in repos_lengths {
            self.add_stock_length(repos_length);
        }
        self
    }

    pub fn add_cut_length(&mut self, demand_length: f64) -> &mut Self {
        let demand_piece = DemandPiece {
            id: self.demand_pieces.len(),
            length: self.length_to_usize(demand_length),
        };
        self.demand_pieces.push(demand_piece);
        self
    }

    pub fn add_cut_lengths(&mut self, demand_lengths: &[f64]) -> &mut Self {
        for &demand_length in demand_lengths {
            self.add_cut_length(demand_length);
        }
        self
    }

    pub fn set_blade_width(&mut self, blade_width: f64) -> &mut Self {
        self.spacing = self.length_to_usize(blade_width);
        self
    }

    pub fn set_random_seed(&mut self, seed: u64) -> &mut Self {
        self.random_seed = seed;
        self
    }

    fn length_to_usize(&self, length: f64) -> usize {
        (length * self.multiplier) as usize
    }

    fn length_to_f64(&self, length: usize) -> f64 {
        length as f64 / self.multiplier
    }

    pub fn optimize(&self) -> Result<Solution, ()> {
        const POPULATION_SIZE: usize = 100;

        let units: Vec<BinPackerUnit> = generate_random_units(
            &self.repos_lengths,
            &self.demand_pieces,
            self.spacing,
            POPULATION_SIZE,
            self.random_seed,
        )
        .ok_or(())?;

        let mut result_units = Population::new(units)
            .set_size(POPULATION_SIZE)
            .set_rand_seed(self.random_seed)
            .set_breed_factor(0.3)
            .set_survival_factor(0.5)
            // .epochs_parallel(1000, 4) // 4 CPU cores
            .epochs(1000)
            .finish();

        let best_unit = &mut result_units[0];

        let mut used_repos_pieces = Vec::with_capacity(best_unit.bins.len());
        for bin in best_unit.bins.iter_mut() {
            bin.demand_pieces.sort_by(|a, b| b.length.cmp(&a.length));
            let mut used_demand_pieces = Vec::with_capacity(bin.demand_pieces.len());
            let mut location = 0;
            for demand_piece in &bin.demand_pieces {
                let used_demand_piece = CutPiece {
                    location: self.length_to_f64(location),
                    length: self.length_to_f64(demand_piece.length),
                };
                used_demand_pieces.push(used_demand_piece);
                location += demand_piece.length + bin.spacing;
            }
            let used_repos_piece = StockPiece {
                length: self.length_to_f64(bin.length),
                demand_pieces: used_demand_pieces,
            };
            used_repos_pieces.push(used_repos_piece);
        }

        Ok(Solution {
            fitness: best_unit.fitness(),
            repos_pieces: used_repos_pieces,
        })
    }
}

pub struct Solution {
    pub fitness: f64,
    pub repos_pieces: Vec<StockPiece>,
}

pub struct StockPiece {
    pub length: f64,
    pub demand_pieces: Vec<CutPiece>,
}

pub struct CutPiece {
    pub location: f64,
    pub length: f64,
}

fn generate_random_units<'a, 'b>(
    repos_lengths: &'b [usize],
    demand_pieces: &'a [DemandPiece],
    spacing: usize,
    num_units: usize,
    random_seed: u64,
) -> Option<Vec<BinPackerUnit<'a, 'b>>> {
    let mut rng: StdRng = SeedableRng::seed_from_u64(random_seed);
    let mut demand_piece_refs: Vec<&DemandPiece> = demand_pieces.iter().collect();
    let mut units = Vec::with_capacity(num_units);
    for _ in 0..num_units {
        demand_piece_refs.shuffle(&mut rng);
        units.push(BinPackerUnit::new(
            repos_lengths,
            &demand_piece_refs,
            spacing,
            &mut rng,
        )?);
    }
    Some(units)
}
