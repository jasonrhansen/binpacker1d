use binpacker1d::optimizer::*;

const RANDOM_SEED: u64 = 100;

fn main() {
    {
        let stock_lengths: Vec<f64> = vec![170.0, 86.5, 50.0, 96.0, 120.0, 90.45, 80.21];
        let demand_lengths: Vec<f64> = vec![
            40.15, 40.0, 170.0, 145.0, 45.34, 20.1, 30.0, 40.5, 50.89, 60.1, 10.0, 40.5, 10.9,
            10.8, 10.7, 20.1, 20.2, 20.3, 30.4, 80.55, 60.67, 30.9, 90.43, 1.0, 2.0, 3.0, 4.0, 5.0,
            6.0, 7.0, 8.0,
        ];
        let blade_width = 0.25;
        optimize_and_print_results(&stock_lengths, &demand_lengths, blade_width);
    }

    {
        let stock_lengths: Vec<f64> = vec![96.0, 120.0];
        let demand_lengths: Vec<f64> = vec![
            10.0, 20.0, 30.0, 40.0, 20.0, 60.0, 60.0, 50.0, 70.0, 16.0, 80.0, 10.0, 20.0, 10.0,
            20.0, 30.0, 3.0, 3.0, 96.0, 24.0, 96.0, 120.0, 60.0, 36.0,
        ];
        let blade_width = 0.0;
        optimize_and_print_results(&stock_lengths, &demand_lengths, blade_width);
    }
}

fn optimize_and_print_results(repos_lengths: &[f64], demand_lengths: &[f64], blade_width: f64) {
    let solution = Optimizer::new(2)
        .add_stock_lengths(repos_lengths)
        .add_cut_lengths(demand_lengths)
        .set_blade_width(blade_width)
        .set_random_seed(RANDOM_SEED)
        .optimize();

    if let Ok(solution) = solution {
        println!("Optimization fitness: {}\n", solution.fitness);
        for repos_piece in solution.repos_pieces {
            println!("Repository piece: {}", repos_piece.length);
            for demand_piece in repos_piece.demand_pieces {
                println!(
                    "\tloc: {: <10}len: {}",
                    demand_piece.location, demand_piece.length
                );
            }
            println!("");
        }
    } else {
        eprintln!("Error running optimizer!");
    }
}
