use core::f64;

use rand::random;

const UP: &str = "+";
const DOWN: &str = "-";
const J: f64 = 10.0; // Interactivity. J > 0 ferromagnetic. J = 0 non-interactive. J < 0
                     // anti-ferromagnetic
const TEMP: f64 = 10000.0; // In K
const KB: f64 = 1.380649e-23; // Boltzmann Constant in J K^-1

struct Lattice {
    value: Vec<Vec<i32>>,
    size: usize,
}

impl Lattice {
    fn new(size: usize) -> Lattice {
        let mut lattice: Vec<Vec<i32>> = Vec::new();
        for _ in 0..size {
            let y_vector = (0..size)
                .map(|_| rand::random_range(0..=1))
                // Alter 0 to -1 (negative spin)
                .map(|s| if &s == &0 { -1 } else { 1 })
                .collect();
            lattice.push(y_vector)
        }
        Lattice {
            value: lattice,
            size,
        }
    }

    fn show(&self) {
        for y_index in 0..self.size {
            for x_index in 0..self.size {
                let spin = if self.value[y_index][x_index] == 1 {
                    UP
                } else {
                    DOWN
                };
                if x_index == self.size - 1 {
                    // Add a new line
                    println!(" {} ", spin)
                } else {
                    // Only print spin
                    print!(" {} ", spin)
                }
            }
        }
    }

    fn pick_random_point(&self) -> (usize, usize) {
        (
            rand::random_range(0..self.size),
            rand::random_range(0..self.size),
        )
    }

    fn calculate_hamiltonian(&self, x_rand: usize, y_rand: usize) -> f64 {
        let random_spin = f64::from(self.value[y_rand][x_rand]);
        println!("Random Point at {}, {}: {}", x_rand, y_rand, random_spin);

        let left = if x_rand != 0 {
            self.value[y_rand][x_rand - 1]
        } else {
            self.value[y_rand][x_rand]
        };
        let right = if x_rand != self.size - 1 {
            self.value[y_rand][x_rand + 1]
        } else {
            self.value[y_rand][x_rand]
        };
        let down = if y_rand != 0 {
            self.value[y_rand - 1][x_rand]
        } else {
            self.value[y_rand][x_rand]
        };
        let up = if y_rand != self.size - 1 {
            self.value[y_rand + 1][x_rand]
        } else {
            self.value[y_rand][x_rand]
        };
        println!(
            "Left: {}, Right: {}, Down: {}, Up: {}.",
            left, right, up, down
        );
        // Hamiltonian Formula
        // H = -J * sum_over_nearest_neighbors(spin_i, spin_j)
        // H = -J * current_spin * sum_of_all_neighbors
        -1.0 * J * random_spin * f64::from(left + right + down + up)
    }

    fn metropolis_algo_calculation(&mut self, x_rand: usize, y_rand: usize) {
        // Delta_H = H_new - H_current
        // Beta = 1 / ( k_B * T)
        // If Delta_H < 0; take the new flip. It's mean the atom transition to a lower energy state
        // If Delta_H > 0;
        // If P(Delta_H) > e^(-Beta * Delta_H); take the new flip. It's mean the atom try to escape
        // a local minima.
        // Else keep the old spin
        let current_hamiltonian_energy = self.calculate_hamiltonian(x_rand, y_rand);
        let new_hamiltonian_energy = -1.0 * current_hamiltonian_energy;
        println!(
            "New and Current H: {}, {}",
            new_hamiltonian_energy, current_hamiltonian_energy
        );

        let delta_h = new_hamiltonian_energy - current_hamiltonian_energy;
        let acceptence_criteria = f64::consts::E.powf(delta_h * (-1.0 / KB * TEMP));
        println!("Delta H: {}", delta_h);
        println!("A: {}", acceptence_criteria);

        // Flip only in these two condition
        if delta_h < 0.0 || acceptence_criteria > 0.5 {
            self.value[y_rand][x_rand] = self.value[y_rand][x_rand] * -1;
            println!("FLIPPED!")
        }
    }
}

fn main() {
    let mut lattice = Lattice::new(10);
    lattice.show();
    let (x_rand, y_rand) = lattice.pick_random_point();
    lattice.metropolis_algo_calculation(x_rand, y_rand);
    lattice.show();

    // // Setup lattice 1D
    // let mut lattice: Vec<i32> = (0..100)
    //     .map(|_| rand::random_range(0..=1))
    //     .map(|s| if &s == &0 { -1 } else { 1 })
    //     .collect();
    //
    // // print initial lattice
    // for spin in &lattice {
    //     match spin {
    //         -1 => print!("{}", DOWN),
    //         1 => print!("{}", UP),
    //         _ => print!("Invalid spin"),
    //     }
    // }
    //
    // // Select random lattice spin
    // let lattice_x_index = rand::random_range(0..100);
    // let spin_selected = &lattice[lattice_x_index];
    //
    // println!("");
    // println!("---");
    // println!(
    //     "Selected Spin index, value: {}, {}",
    //     &lattice_x_index, &spin_selected
    // );
    //
    // // Calculate neighbor sum of nearby random lattice
    // let mut neighbor_sum = 0;
    // if lattice_x_index >= 0 && lattice_x_index < lattice.len() {
    //     neighbor_sum = neighbor_sum + lattice[lattice_x_index + 1]; // Right
    //     neighbor_sum = neighbor_sum + lattice[lattice_x_index - 1]; // Left
    // } else if lattice_x_index >= lattice.len() {
    //     // Hit right boundary
    //     neighbor_sum = neighbor_sum + lattice[lattice_x_index - 1]; // Left
    // } else if lattice_x_index < 0 {
    //     // Hit left boundary
    //     neighbor_sum = neighbor_sum + lattice[lattice_x_index + 1]; // Right
    // }
    //
    // println!("---");
    // println!("Neighbor Sum: {}", neighbor_sum);
    // println!(
    //     "Neighbor Right and Left values: {}, {}",
    //     lattice[lattice_x_index + 1],
    //     lattice[lattice_x_index - 1]
    // );
    //
    // // Setup constant
    // let interaction = 1;
    //
    // // Calculate hamiltonian delta energy
    // // del_E = 2 * old_spin * (interaction * neighbor_sum)
    // let delta_energy = 2 * spin_selected * (interaction * neighbor_sum);
    // let temperature = 1.0;
    //
    // println!("---");
    // println!("Delta Energy: {}", delta_energy);
    //
    // // Setup Metropolis Criterion
    // if delta_energy > 0 {
    //     let prob_accept = std::f64::consts::E.powf(-1.0 * f64::from(delta_energy) / temperature);
    //     let r = random::<f64>();
    //
    //     if prob_accept > r {
    //         println!("FLIP ON INDEX {}!", &lattice_x_index);
    //         lattice[lattice_x_index] = -1 * lattice[lattice_x_index];
    //     } else {
    //         println!("Skip Flip");
    //     }
    // } else {
    //     println!("FLIP ON INDEX {}!", &lattice_x_index);
    //     lattice[lattice_x_index] = -1 * lattice[lattice_x_index];
    // }
    //
    // // print lattice after flip
    // for spin in &lattice {
    //     match spin {
    //         -1 => print!("{}", DOWN),
    //         1 => print!("{}", UP),
    //         _ => print!("Invalid spin"),
    //     }
    // }
}
