use core::f64;
const KB: f64 = 1.380_649e-23; // Boltzmann Constant in J K^-1

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Spins {
    pub value: Vec<i32>,
}

impl Spins {
    // Create a new random spin vector with value of -1 or 1
    fn new(size: usize) -> Self {
        Self {
            value: (0..size)
                // Generate random spins
                .map(|_| rand::random_bool(0.5))
                .map(|up| if up { 1 } else { -1 })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Lattice {
    /// the 2d lattice
    pub value: Vec<Spins>,
    /// lattice size
    pub size: usize,
    /// sim interactivity
    pub interactivity: f64,
    /// sim temperature
    pub temperature: f64,
}

impl Lattice {
    /// Create a new Lattice with provided size, interactivity, and temperature
    #[must_use]
    pub fn new(size: usize, interactivity: f64, temperature: f64) -> Self {
        let mut value: Vec<Spins> = Vec::new();
        for _ in 0..size {
            let spins = Spins::new(size);
            value.push(spins);
        }
        Self {
            value,
            size,
            interactivity,
            temperature,
        }
    }

    /// # Update Lattice when a new size configured
    /// If desire `size` is the same as the current `size`, nothing happen
    /// If it's greater, append new spin on the current spin columns then append the spins
    /// If it's lesser, delete the outer layer spin columns then pop the outer layer spins
    ///
    /// # Panics
    ///
    /// Will panic while `poping` the outer layer of spin columns if return `None`
    #[must_use]
    pub fn update_lattice(&mut self) -> Self {
        match self.size.cmp(&self.value.len()) {
            // if diff == 0 return early
            std::cmp::Ordering::Equal => {
                return self.clone();
            }
            // if diff > 0
            // add new values based on the difference
            std::cmp::Ordering::Greater => {
                let diff = self.size - self.value.len();
                // Add new values to existing spins vector
                for spins in &mut self.value {
                    let mut new_spins = Spins::new(diff);
                    spins.value.append(&mut new_spins.value);
                }
                // Add new spins vector to lattice value
                for _spins_id in 0..diff {
                    let new_spins_vector = Spins::new(self.size);
                    self.value.push(new_spins_vector);
                }
            }
            // else if diff < 0
            // decrease outer values based on the difference
            std::cmp::Ordering::Less => {
                let diff = self.value.len() - self.size;
                // Delete the outer values in the spins
                for spins in &mut self.value {
                    for _del_occ in 0..diff.abs_diff(0) {
                        let _ = spins.value.pop().unwrap();
                    }
                }
                // Delete the existing outer spins
                for _spins_id in 0..diff {
                    self.value.pop();
                }
            }
        }
        self.clone()
    }

    #[must_use]
    pub fn reset_value(&self) -> Self {
        Self::new(self.size, self.interactivity, self.temperature)
    }

    /// Set Lattice Size
    #[must_use]
    pub fn set_size(&mut self, size: usize) -> Self {
        if size > 0 {
            self.size = size;
        }
        self.clone()
    }

    /// pick randomg x and y point to be sampled
    #[must_use]
    pub fn pick_random_point(&self) -> (usize, usize) {
        (
            rand::random_range(0..self.size),
            rand::random_range(0..self.size),
        )
    }

    /// Hamiltonian Formula
    /// H = -J * `sum_over_nearest_neighbors`(`spin_i`, `spin_j`)
    /// H = -J * `current_spin` * `sum_of_all_neighbors`
    #[must_use]
    pub fn calculate_hamiltonian(&self, x_rand: usize, y_rand: usize) -> f64 {
        let current_spin = f64::from(self.value[y_rand].value[x_rand]);
        let (left, right, down, up) = self.find_neighbours(x_rand, y_rand);

        -self.interactivity * current_spin * f64::from(left + right + down + up)
    }

    /// Gather nearest neighbours
    #[must_use]
    pub fn find_neighbours(&self, x_rand: usize, y_rand: usize) -> (i32, i32, i32, i32) {
        let current_spin = self.value[y_rand].value[x_rand];
        let is_not_most_left = x_rand != 0;
        let is_not_most_right = x_rand != self.size - 1;
        let is_not_bottom = y_rand != 0;
        let is_not_top = y_rand != self.size - 1;

        let (mut left, mut right, mut down, mut up) =
            (current_spin, current_spin, current_spin, current_spin);

        if is_not_most_left {
            left = self.value[y_rand].value[x_rand - 1];
        }
        if is_not_most_right {
            right = self.value[y_rand].value[x_rand + 1];
        }
        if is_not_bottom {
            down = self.value[y_rand - 1].value[x_rand];
        }
        if is_not_top {
            up = self.value[y_rand + 1].value[x_rand];
        }

        (left, right, down, up)
    }

    /// Metropolis Algorith Calculation
    /// If `Delta_H` < 0; take the new flip. It's mean the atom transition to a lower energy state
    /// If `Delta_H` > 0;
    /// If Acceptence Criteria > 0.5; take the new flip. It's mean the atom try to escape
    /// a local minima.
    /// Else keep the old spin
    pub fn metropolis_algo_calculation(&mut self, x_rand: usize, y_rand: usize) {
        let delta_h = self.calculate_delta_h(x_rand, y_rand);
        let acceptence_criteria = self.calculate_acceptence_criteria(delta_h);

        // Flip only when delta H is lower than 0 and acceptence_criteria is higher than half
        // Half represent the threshold to flip or not
        let is_flipped = delta_h < 0.0 || acceptence_criteria > 0.5;
        if is_flipped {
            self.value[y_rand].value[x_rand] = -self.value[y_rand].value[x_rand];
        }
    }

    /// Calculate Hamiltonian energy difference of a point
    /// `Delta_H` = `H_new` - `H_current`
    pub fn calculate_delta_h(&mut self, x: usize, y: usize) -> f64 {
        let current_hamiltonian_energy = self.calculate_hamiltonian(x, y);
        let flipped_hamiltonian_energy = -current_hamiltonian_energy;
        flipped_hamiltonian_energy - current_hamiltonian_energy
    }

    /// Beta = 1 / ( `k_B` * T)
    /// Acceptence Criteria = e^(-Beta * `Delta_H`)
    pub fn calculate_acceptence_criteria(&mut self, delta_h: f64) -> f64 {
        let minus_beta = -1.0 / (KB * self.temperature);
        let acceptence_criteria = (minus_beta * delta_h).exp();
        if acceptence_criteria == f64::NAN {
            return 0.0;
        }
        match acceptence_criteria {
            f64::INFINITY => 1.0,
            res => res,
        }
    }
}
