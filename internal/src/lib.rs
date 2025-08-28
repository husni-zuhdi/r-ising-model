use core::f64;
const KB: f64 = 1.380649e-23; // Boltzmann Constant in J K^-1

#[derive(Clone, Debug, Default)]
pub struct Lattice {
    // the 3d lattice
    pub value: Vec<Vec<i32>>,
    // lattice size
    pub size: usize,
    pub interactivity: f64,
    pub temperature: f64,
}

impl Lattice {
    pub fn new(size: usize, interactivity: f64, temperature: f64) -> Lattice {
        let mut lattice: Vec<Vec<i32>> = Vec::new();
        for _ in 0..size {
            let y_vector = (0..size)
                .map(|_| rand::random_range(0..=1))
                // Alter 0 to -1 (negative spin)
                .map(|s| if s == 0 { -1 } else { 1 })
                .collect();
            lattice.push(y_vector)
        }
        Lattice {
            value: lattice,
            size,
            interactivity,
            temperature,
        }
    }

    // convert 1 and 0 to String
    // TODO: I don't think we need to do this. Can we cast int -> Line ?
    pub fn convert_to_string(self) -> Vec<Vec<String>> {
        let mut lattice: Vec<Vec<String>> = Vec::new();

        for y in 0..self.size {
            let mut y_vector: Vec<String> = Vec::new();
            for x in 0..self.size {
                y_vector.push(self.value[y][x].to_string());
            }
            lattice.push(y_vector);
        }

        lattice
    }

    // pick randomg x and y point to be sampled
    pub fn pick_random_point(&self) -> (usize, usize) {
        (
            rand::random_range(0..self.size),
            rand::random_range(0..self.size),
        )
    }

    // Hamiltonian Formula
    // H = -J * sum_over_nearest_neighbors(spin_i, spin_j)
    // H = -J * current_spin * sum_of_all_neighbors
    pub fn calculate_hamiltonian(&self, x_rand: usize, y_rand: usize) -> f64 {
        let current_spin = f64::from(self.value[y_rand][x_rand]);
        let (left, right, down, up) = self.find_neighbours(x_rand, y_rand);

        -1.0 * self.interactivity * current_spin * f64::from(left + right + down + up)
    }

    // Gather nearest neighbours
    pub fn find_neighbours(&self, x_rand: usize, y_rand: usize) -> (i32, i32, i32, i32) {
        let current_spin = self.value[y_rand][x_rand];
        let is_not_most_left = x_rand != 0;
        let is_not_most_right = x_rand != self.size - 1;
        let is_not_bottom = y_rand != 0;
        let is_not_top = y_rand != self.size - 1;

        let (mut left, mut right, mut down, mut up) =
            (current_spin, current_spin, current_spin, current_spin);

        if is_not_most_left {
            left = self.value[y_rand][x_rand - 1]
        };
        if is_not_most_right {
            right = self.value[y_rand][x_rand + 1]
        };
        if is_not_bottom {
            down = self.value[y_rand - 1][x_rand]
        };
        if is_not_top {
            up = self.value[y_rand + 1][x_rand]
        };

        (left, right, down, up)
    }

    // Delta_H = H_new - H_current
    // Beta = 1 / ( k_B * T)
    // If Delta_H < 0; take the new flip. It's mean the atom transition to a lower energy state
    // If Delta_H > 0;
    // If P(Delta_H) > e^(-Beta * Delta_H); take the new flip. It's mean the atom try to escape
    // a local minima.
    // Else keep the old spin
    pub fn metropolis_algo_calculation(&mut self, x_rand: usize, y_rand: usize) {
        let current_hamiltonian_energy = self.calculate_hamiltonian(x_rand, y_rand);
        let flipped_hamiltonian_energy = -1.0 * current_hamiltonian_energy;

        let delta_h = flipped_hamiltonian_energy - current_hamiltonian_energy;
        let minus_beta = -1.0 / (KB * self.temperature);
        let acceptence_criteria = f64::consts::E.powf(minus_beta * delta_h);

        // Flip only when delta H is lower than 0 and acceptence_criteria is higher than half
        // Half represent the threshold to flip or not
        let is_flipped = delta_h < 0.0 || acceptence_criteria > 0.5;
        if is_flipped {
            self.value[y_rand][x_rand] = -self.value[y_rand][x_rand];
        }
    }
}
