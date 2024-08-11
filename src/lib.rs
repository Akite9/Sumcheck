use std::collections::HashMap;
use std::ops::Add;
use std::io;


// Helper function to perform modular exponentiation (base^exp % modulus)
fn modular_pow(base: i32, exp: usize, modulus: i32) -> i32 {
    let mut result = 1;
    let mut base = base.rem_euclid(modulus);
    let mut exp = exp;

    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base).rem_euclid(modulus);
        }
        base = (base * base).rem_euclid(modulus);
        exp /= 2;
    }
    result
}

// Helper function to check if the modulus is prime
fn is_prime(num: i32) -> bool {
    if num <= 1 {
        return false;
    }
    if num <= 3 {
        return true;
    }
    if num % 2 == 0 || num % 3 == 0 {
        return false;
    }
    let mut i = 5;
    while i * i <= num {
        if num % i == 0 || num % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

// Define a struct for multi-variable polynomials
#[derive(Debug, Clone, PartialEq)]
pub struct MultiVarPolynomial {
    pub terms: HashMap<Vec<usize>, i32>, // Map from exponents to coefficients
    pub num_vars: usize, // Number of variables
    pub modulus: i32, // Prime modulus for the finite field
}

impl MultiVarPolynomial {
    // Create a new polynomial with a given number of variables and modulus
    pub fn new(num_vars: usize, modulus: i32) -> Self {
         if modulus <= 0 || !is_prime(modulus) {
            panic!("Modulus must be a positive prime number");
        }
        Self {
            terms: HashMap::new(),
            num_vars,
            modulus,
        }
    }

    // Add a term to the polynomial
    pub fn add_term(&mut self, exponents: Vec<usize>, coefficient: i32) {
        if exponents.len() != self.num_vars {
            panic!("Number of exponents must match the number of variables");
        }
        let reduced_coefficient = coefficient.rem_euclid(self.modulus);
        self.terms
            .entry(exponents)
            .and_modify(|c| *c = (*c + reduced_coefficient).rem_euclid(self.modulus))
            .or_insert(reduced_coefficient);
    }

    // Get the degree of the polynomial with respect to a specific variable
    pub fn degree_in_var(&self, var_index: usize) -> usize {
        if var_index >= self.num_vars {
            panic!("Variable index out of bounds");
        }

        self.terms
            .keys()
            .map(|exponents| exponents[var_index])
            .max()
            .unwrap_or(0)
    }

    // Partially evaluate the polynomial at specific values for given variables
    pub fn partial_eval(&self, values: Vec<(usize, i32)>) -> Self {
        let mut new_terms: HashMap<Vec<usize>, i32> = HashMap::new();

        // Create a map for easy lookup of variable evaluations
        let eval_map: HashMap<usize, i32> = values.into_iter().collect();

        for (exponents, coeff) in &self.terms {
            let mut new_coeff = *coeff;
            let mut new_exponents = Vec::new();

            for (var_index, exp) in exponents.iter().enumerate() {
                if let Some(&value) = eval_map.get(&var_index) {
                    // Apply modular exponentiation and multiplication to avoid overflow
                    let mod_exp = modular_pow(value, *exp, self.modulus);
                    new_coeff = (new_coeff * mod_exp).rem_euclid(self.modulus);
                } else {
                    // Keep the variable in the reduced polynomial
                    new_exponents.push(*exp);
                }
            }

            new_terms
                .entry(new_exponents)
                .and_modify(|c| *c = (*c + new_coeff).rem_euclid(self.modulus))
                .or_insert(new_coeff);
        }

        // Calculate the new number of variables by removing evaluated variables
        let new_num_vars = self.num_vars - eval_map.len();

        MultiVarPolynomial {
            terms: new_terms,
            num_vars: new_num_vars,
            modulus: self.modulus,
        }
    }

    // Function to read polynomial from user input
    pub fn read_from_input() -> Self {
        // Read the number of variables from the user
        println!("Enter the number of variables in the polynomial:");
        let mut num_vars_input = String::new();
        io::stdin()
            .read_line(&mut num_vars_input)
            .expect("Failed to read line");
        let num_vars: usize = num_vars_input.trim().parse().expect("Invalid number");

        // Read the modulus from the user
        println!("Enter the modulus of the finite field (must be a positive prime):");
        let mut modulus_input = String::new();
        io::stdin()
            .read_line(&mut modulus_input)
            .expect("Failed to read line");
        let modulus: i32 = modulus_input.trim().parse().expect("Invalid modulus");

        println!(
            "Enter polynomial terms in the format 'coeff:exp1,exp2,...; coeff:exp1,exp2,...'"
        );

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let input = input.trim();
        let mut polynomial = MultiVarPolynomial::new(num_vars, modulus);

        for term in input.split(';') {
            let parts: Vec<&str> = term.trim().split(':').collect();
            if parts.len() != 2 {
                panic!("Invalid term format");
            }

            let coefficient: i32 = parts[0].trim().parse().expect("Invalid coefficient");
            let exponents: Vec<usize> = parts[1]
                .trim()
                .split(',')
                .map(|e| e.trim().parse().expect("Invalid exponent"))
                .collect();

            if exponents.len() != num_vars {
                panic!("Each term must have the correct number of exponents");
            }

            polynomial.add_term(exponents, coefficient);
        }

        polynomial
    }
      
    // Function to partially calculate bool sum
    pub fn bool_sum(&self) -> Self {
        self.partial_eval(vec![(self.num_vars-1,0)]) + self.partial_eval(vec![(self.num_vars-1,1)])
    }
}

// Add two multi-variable polynomials together
impl Add for MultiVarPolynomial {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.num_vars != other.num_vars {
            panic!("Polynomials must have the same number of variables to be added");
        }

        if self.modulus != other.modulus {
            panic!("Polynomials must be over the same finite field to be added");
        }

        let mut result = self.clone();

        for (exp, coeff) in other.terms {
            result.add_term(exp, coeff);
        }

        result
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_create_polynomial() {
        let poly = MultiVarPolynomial::new(3, 7);
        assert_eq!(poly.num_vars, 3);
        assert_eq!(poly.modulus, 7);
        assert!(poly.terms.is_empty());
    }

    #[test]
    fn test_add_term() {
        let mut poly = MultiVarPolynomial::new(2, 5);
        poly.add_term(vec![1, 2], 3);
        assert_eq!(poly.terms.len(), 1);
        assert_eq!(poly.terms.get(&vec![1, 2]), Some(&3));
        
        // Adding another term with the same exponent
        poly.add_term(vec![1, 2], 2);
        assert_eq!(poly.terms.get(&vec![1, 2]), Some(&0)); // (3 + 2) % 5 = 0
    }

    #[test]
    fn test_degree_in_var() {
        let mut poly = MultiVarPolynomial::new(2, 11);
        poly.add_term(vec![3, 1], 4);
        poly.add_term(vec![1, 2], 5);
        assert_eq!(poly.degree_in_var(0), 3);
        assert_eq!(poly.degree_in_var(1), 2);
    }
    
    #[test]
    fn test_partial_eval() {
        // Create a polynomial in 2 variables: x_1 + x_2
        let mut poly = MultiVarPolynomial::new(2, 23);
        poly.add_term(vec![1, 0], 1); // x_1
        poly.add_term(vec![0, 1], 1); // x_2

        // Partially evaluate polynomial at x_1 = 3
        let partial_eval_poly = poly.partial_eval(vec![(0, 3)]);

        // Expected result: 3 + x_2
        let mut expected_poly = MultiVarPolynomial::new(1, 23);
        expected_poly.add_term(vec![0], 3); // 3 (constant term after x_1 evaluation)
        expected_poly.add_term(vec![1], 1); // x_2

        assert_eq!(partial_eval_poly, expected_poly);
    }

    #[test]
    fn test_bool_sum() {
        let mut poly = MultiVarPolynomial::new(2, 5);
        poly.add_term(vec![1, 0], 2);
        poly.add_term(vec![0, 1], 3);

        let bool_sum_poly = poly.bool_sum();
        let mut expected_terms = HashMap::new();
        expected_terms.insert(vec![1], 4);
        expected_terms.insert(vec![0], 3);

        let expected_poly = MultiVarPolynomial {
            terms: expected_terms,
            num_vars: 1,
            modulus: 5,
        };
        assert_eq!(bool_sum_poly, expected_poly);
    }

    #[test]
    fn test_addition() {
        let mut poly1 = MultiVarPolynomial::new(2, 11);
        poly1.add_term(vec![1, 1], 4);
        poly1.add_term(vec![0, 0], 3);

        let mut poly2 = MultiVarPolynomial::new(2, 11);
        poly2.add_term(vec![1, 1], 5);
        poly2.add_term(vec![0, 1], 2);

        let sum_poly = poly1 + poly2;
        let mut expected_terms = HashMap::new();
        expected_terms.insert(vec![1, 1], 9); // (4 + 5) % 11 = 9
        expected_terms.insert(vec![0, 0], 3);
        expected_terms.insert(vec![0, 1], 2);

        let expected_poly = MultiVarPolynomial {
            terms: expected_terms,
            num_vars: 2,
            modulus: 11,
        };
        assert_eq!(sum_poly, expected_poly);
    }

    #[test]
    fn test_is_prime() {
        // Testing known primes
        assert!(is_prime(8009));
    }

    #[test]
    fn test_modular_pow() {
        // Testing modular exponentiation
        assert_eq!(modular_pow(2, 3, 5), 3); // (2^3 % 5) = 8 % 5 = 3
    }
}