pub mod ffi;
pub mod optimizer;

pub(crate) mod population;
pub(crate) mod unit;

use rand::prelude::*;
use std::borrow::Borrow;
use unit::Unit;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DemandPiece {
    pub(crate) id: usize,
    pub(crate) length: usize,
}

#[derive(Clone)]
pub(crate) struct UsedDemandPiece<'a> {
    demand_piece: &'a DemandPiece,
    location: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct Bin<'a> {
    pub(crate) length: usize,
    pub(crate) demand_pieces: Vec<&'a DemandPiece>,
    spacing: usize,
}

impl<'a> Bin<'a> {
    fn fitness(&self) -> f64 {
        (self.used_length_including_spacing() as f64 / self.length as f64).powf(2.0)
    }

    fn insert(&mut self, demand_piece: &'a DemandPiece) -> Option<usize> {
        let insert_location = if self.demand_pieces.is_empty() {
            0
        } else {
            self.used_length_including_spacing() + self.spacing
        };
        if insert_location <= self.length && self.length - insert_location >= demand_piece.length {
            self.demand_pieces.push(demand_piece);
            Some(insert_location)
        } else {
            None
        }
    }

    fn sum_of_demand_lengths(&self) -> usize {
        self.demand_pieces.iter().fold(0, |acc, p| acc + p.length)
    }

    fn used_length_including_spacing(&self) -> usize {
        let length = self.sum_of_demand_lengths();
        if self.demand_pieces.len() > 1 {
            length + (self.spacing * (self.demand_pieces.len() - 1))
        } else {
            length
        }
    }

    fn remove_demand_pieces<I>(&mut self, demand_pieces: I) -> usize
    where
        I: Iterator,
        I::Item: Borrow<DemandPiece> + 'a,
    {
        let old_len = self.demand_pieces.len();
        for demand_piece_to_remove in demand_pieces {
            self.demand_pieces
                .retain(|demand_piece| *demand_piece != demand_piece_to_remove.borrow());
        }
        old_len - self.demand_pieces.len()
    }
}

#[derive(Debug)]
pub(crate) struct BinPackerUnit<'a, 'b> {
    pub(crate) bins: Vec<Bin<'a>>,
    possible_bin_lengths: &'b [usize],
    spacing: usize,
}

impl<'a, 'b> BinPackerUnit<'a, 'b> {
    pub(crate) fn new<R>(
        repos_lengths: &'b [usize],
        demand_pieces: &[&'a DemandPiece],
        spacing: usize,
        rng: &mut R,
    ) -> Option<BinPackerUnit<'a, 'b>>
    where
        R: Rng + ?Sized,
    {
        let mut unit = BinPackerUnit {
            bins: Vec::new(),
            possible_bin_lengths: repos_lengths,
            spacing,
        };

        for demand_piece in demand_pieces {
            unit.first_fit(demand_piece, rng)?;
        }

        Some(unit)
    }

    fn first_fit<R>(&mut self, demand_piece: &'a DemandPiece, rng: &mut R) -> Option<usize>
    where
        R: Rng + ?Sized,
    {
        for bin in self.bins.iter_mut() {
            if let Some(location) = bin.insert(demand_piece) {
                return Some(location);
            }
        }

        self.add_to_new_bin(demand_piece, rng)
    }

    fn add_to_new_bin<R>(&mut self, demand_piece: &'a DemandPiece, rng: &mut R) -> Option<usize>
    where
        R: Rng + ?Sized,
    {
        let possible_lengths: Vec<usize> = self
            .possible_bin_lengths
            .iter()
            .filter(|length| **length >= demand_piece.length)
            .map(|&length| length)
            .collect();

        if let Some(&length) = possible_lengths.choose(rng) {
            let bin = Bin {
                length,
                demand_pieces: vec![demand_piece],
                spacing: self.spacing,
            };
            self.bins.push(bin);
            Some(0)
        } else {
            None
        }
    }

    fn crossover<R>(&self, other: &BinPackerUnit<'a, 'b>, rng: &mut R) -> BinPackerUnit<'a, 'b>
    where
        R: Rng + ?Sized,
    {
        let cross_dest = rng.gen_range(0, self.bins.len());
        let cross_src_start = rng.gen_range(0, other.bins.len());
        let cross_src_end = rng.gen_range(cross_src_start, other.bins.len());

        let mut new_unit = BinPackerUnit {
            // Inject bins between crossing sites of other.
            bins: (&self.bins[..cross_dest])
                .iter()
                .chain((&other.bins[cross_src_start..cross_src_end]).iter())
                .chain((&self.bins[cross_dest..]).iter())
                .cloned()
                .collect(),
            possible_bin_lengths: self.possible_bin_lengths,
            spacing: self.spacing,
        };

        let mut removed_demand_pieces: Vec<&DemandPiece> = Vec::new();
        for i in (0..cross_dest)
            .chain(cross_dest + cross_src_end - cross_src_start..new_unit.bins.len())
            .rev()
        {
            let bin = &mut new_unit.bins[i];
            let injected_demand_pieces = (&other.bins[cross_src_start..cross_src_end])
                .iter()
                .flat_map(|b| b.demand_pieces.iter())
                .map(|&demand_piece| demand_piece);
            if bin.remove_demand_pieces(injected_demand_pieces) > 0 {
                for demand_piece in &bin.demand_pieces {
                    removed_demand_pieces.push(demand_piece);
                }
                new_unit.bins.remove(i);
            }
        }

        for demand_piece in &removed_demand_pieces {
            new_unit.first_fit(demand_piece, rng);
        }

        new_unit
    }

    // Randomly apply a mutation to this unit.
    fn mutate<R>(&mut self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        match rng.gen_range(0, 20) {
            0 => self.elimination(rng),
            1 => self.inversion(rng),
            _ => (),
        }
    }

    // Remove least-fit bin and re-insert demand pieces in random order.
    fn elimination<R>(&mut self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let mut worst_fitness = 1.0;
        let mut worst_idx = 0;
        for (i, bin) in self.bins.iter().enumerate() {
            let fitness = bin.fitness();
            if fitness < worst_fitness {
                worst_fitness = fitness;
                worst_idx = i;
            }
        }

        if worst_fitness < 1.0 {
            let mut worst_bin = self.bins.remove(worst_idx);
            worst_bin.demand_pieces.shuffle(rng);
            for demand_piece in worst_bin.demand_pieces {
                self.first_fit(demand_piece, rng);
            }
        }
    }

    // Reverse order of a random range of bins.
    fn inversion<R>(&mut self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let start = rng.gen_range(0, self.bins.len());
        let end = rng.gen_range(start, self.bins.len());
        &self.bins[start..end].reverse();
    }
}

impl<'a, 'b> Unit for BinPackerUnit<'a, 'b> {
    fn fitness(&self) -> f64 {
        self.bins.iter().fold(0.0, |acc, b| acc + b.fitness()) / self.bins.len() as f64
    }

    fn breed_with<R>(&self, other: &BinPackerUnit<'a, 'b>, rng: &mut R) -> BinPackerUnit<'a, 'b>
    where
        R: Rng + ?Sized,
    {
        let mut new_unit = self.crossover(other, rng);
        new_unit.mutate(rng);
        new_unit
    }
}
