use rand::thread_rng;
use rand::Rng;
use Sumcheck::MultiVarPolynomial;
use std::collections::HashMap;
use std::io;

fn main() {
    // Read a polynomial from user input
    let polynomial = MultiVarPolynomial::read_from_input();

    // Read prover overrides from user input
    let prover_overrides = read_prover_overrides(polynomial.num_vars, polynomial.modulus);

    // Read verifier overrides from user input
    let verifier_overrides = read_verifier_overrides(polynomial.num_vars, polynomial.modulus);

    // Run the protocol
    if let Err(e) = run_protocol(polynomial, prover_overrides, verifier_overrides) {
        eprintln!("Protocol failed: {}", e);
    }
}

// Helper function to read prover overrides from user input
fn read_prover_overrides(num_vars: usize, modulus: i32) -> HashMap<usize, MultiVarPolynomial> {
    let mut overrides = HashMap::new();

    loop {
        println!("Enter the j of the g_j you want to override, or press enter to skip:");
        let mut var_index_input = String::new();
        io::stdin().read_line(&mut var_index_input).expect("Failed to read line");

        let var_index_input = var_index_input.trim();

        // Break loop if the user presses enter without input
        if var_index_input.is_empty() {
            break;
        }

        let var_index: usize = var_index_input.parse().expect("Invalid variable index");

        if var_index < 1 || var_index > num_vars {
            panic!("Variable index out of bounds");
        }

        let polynomial = read_single_prover_override(var_index, modulus);
        overrides.insert(var_index, polynomial);
    }

    overrides
}

fn read_single_prover_override(var_index: usize, modulus: i32) -> MultiVarPolynomial {
    // Read the polynomial from input
    let polynomial = MultiVarPolynomial::read_from_input();

    // Check that the polynomial is univariate
    if polynomial.num_vars != 1 {
        panic!("g_{var_index} must be univariate");
    }

    // Check that the polynomial has the correct modulus
    if polynomial.modulus != modulus {
        panic!("g_{var_index} must have the same modulus as the original polynomial");
    }

    polynomial
}

// Helper function to read verifier overrides from user input
fn read_verifier_overrides(num_vars: usize, modulus: i32) -> HashMap<usize, i32> {
    let mut overrides = HashMap::new();
    let mut input = String::new();
        
    println!("Enter verifier overrides in the format 'var_index:value; var_index:value', or press enter to skip:");

    io::stdin().read_line(&mut input).expect("Failed to read line");
        
    let input = input.trim();
        
    if input.is_empty() {
        return overrides;
    }

    for entry in input.split(';') {
        let parts: Vec<&str> = entry.trim().split(':').collect();
        if parts.len() != 2 {
            panic!("Invalid format for verifier override. Expected 'var_index:value'");
        }

        let var_index: usize = parts[0].trim().parse().expect("Invalid variable index");
        let value: i32 = parts[1].trim().parse().expect("Invalid value");

        if var_index < 1 || var_index > num_vars {
            panic!("Value for variable index must be between 1 and {num_vars}")
        }
        
        if value < 0 || value >= modulus {
            panic!("Value for r_{var_index} must be within the finite field defined by modulus {}", modulus);
        }

        overrides.insert(var_index, value);
    }

    overrides
}
    
fn run_protocol(
    polynomial: MultiVarPolynomial,
    prover_overrides: HashMap<usize, MultiVarPolynomial>, // Maps num_var to g_j override
    verifier_overrides: HashMap<usize, i32>,              // Maps num_var to r_j override
) -> Result<(), String> {
    println!("Parsed polynomial: {:?}", polynomial);    

    //Setup the random number generator
    let mut rng = thread_rng();
    let mut values = vec![];

    //Prover calculates C and sends to verifier
    let c = compute_g_j(&polynomial, 0, values.clone());
    println!("C is: {:?}", c);

    let mut g_prev = c;

    for num_var in 1..=polynomial.num_vars { 
        // Verifier selects or overrides random element of the field to send to prover
        let r = if num_var > 1 {
            let r = if let Some(&override_r) = verifier_overrides.get(&(num_var - 1)) {
                override_r
            }
            else {
               rng.gen_range(0..polynomial.modulus)
            };
            values.push((num_var - 2, r));
            println!("r_{} is: {r}", num_var - 1);
            r
        }
        else {
            0
        };
        
        // Prover calculates or overrides g_j(X_j) to send to verifier
        let g = if let Some(override_g) = prover_overrides.get(&num_var) {
            override_g.clone()
        } else {
            compute_g_j(&polynomial, 1, values.clone())
        };
        println!("g_{num_var} is: {:?}", g);

        //Verifier checks g_j is a polynomial in 1 var, rejecting if not
         if g.num_vars != 1 {
            return Err(format!("Proof rejected as g_{num_var} is not univariate"));
        }

         //Verifier checks degree g_j(X_j) <= deg_j(g), rejecting if not
        if g.degree_in_var(0) > polynomial.degree_in_var(num_var-1)  {
            return Err(format!("Proof rejected for degree reasons for g_{num_var}!"));
        }

        //Verifier checks g_{j-1}(r_{j-1}) = g_j(0) + g_j(1), rejecting if not
        if  g_prev.partial_eval(if num_var > 1 {vec![(0, r)]} else {vec![]}) != g.bool_sum() {
            return Err(format!("Proof rejected as g_{}(r_{}) != g_{num_var}(0) + g_{num_var}(1)!", num_var-1, num_var-1));
        }

        g_prev = g;
    }

    //Finally verifier picks last element
    let r = if let Some(&override_r) = verifier_overrides.get(&(polynomial.num_vars)) {
        override_r
    } else {
        rng.gen_range(0..polynomial.modulus)
    };
    values.push((polynomial.num_vars - 1, r));
    println!("r_{} is: {:?}", polynomial.num_vars, r);

    //Verifier checks g(r_1, ..., r_n) = g_n(r_n), rejecting if not
    if polynomial.partial_eval(values) != g_prev.partial_eval(vec![(0,r)]) {
        return Err(format!("Proof rejected by final check!"));
    }

    println!("Proof accepted!");
    Ok(())

}

// Function to compute g_j polynomial by partially evaluating and then applying Boolean sum reduction
fn compute_g_j(poly : &MultiVarPolynomial, num_remaining_vars : usize, values: Vec<(usize, i32)>) -> MultiVarPolynomial {
    // Start with partial evaluation based on provided values
    let mut reduced_poly = if !values.is_empty() {
        poly.partial_eval(values)
    } else {
        poly.clone()
    };

    // Apply Boolean sum reduction until the desired number of variables is achieved
    while reduced_poly.num_vars > num_remaining_vars {
        reduced_poly = reduced_poly.bool_sum();
    }

    reduced_poly
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_run_protocol_with_success() {
        // Define a polynomial in 3 variables: 2*x_1^3 + x_1*x_3 + x_2*x_3
        let mut polynomial = MultiVarPolynomial::new(3, 97);
        polynomial.add_term(vec![3, 0, 0], 2);
        polynomial.add_term(vec![1, 0, 1], 1);
        polynomial.add_term(vec![0, 1, 1], 1);

        // Override all r_j values
        let mut verifier_overrides: HashMap<usize, i32> = HashMap::new();
        verifier_overrides.insert(1, 2);  // r_1 = 2
        verifier_overrides.insert(2, 3);  // r_2 = 3
        verifier_overrides.insert(3, 6);  // r_3 = 1

        // No prover overrides in this case
        let prover_overrides: HashMap<usize, MultiVarPolynomial> = HashMap::new();

        // Run the protocol and check if it succeeds
        let result = run_protocol(polynomial, prover_overrides, verifier_overrides);

        assert!(result.is_ok(), "The protocol should have succeeded");
    }

    #[test]
    #[should_panic(expected = "Proof rejected")]
    fn test_run_protocol_with_fail() {
        // Define a polynomial in 3 variables: 2*x_1^3 + x_1*x_3 + x_2*x_3
        let mut polynomial = MultiVarPolynomial::new(3, 97);
        polynomial.add_term(vec![3, 0, 0], 2);
        polynomial.add_term(vec![1, 0, 1], 1);
        polynomial.add_term(vec![0, 1, 1], 1);

        // No verifier overrides here
        let verifier_overrides: HashMap<usize, i32> = HashMap::new();

        // Prover overrides g_1 with an incorrect polynomial
        let mut incorrect_g1 = MultiVarPolynomial::new(1, 97);  // Incorrect polynomial in 1 variable
        incorrect_g1.add_term(vec![3], 8);
        incorrect_g1.add_term(vec![1], 2);
        let mut prover_overrides: HashMap<usize, MultiVarPolynomial> = HashMap::new();
        prover_overrides.insert(1, incorrect_g1);  // Override g_1

        // Run the protocol and expect it to panic (fail)
        run_protocol(polynomial, prover_overrides, verifier_overrides).unwrap();
    }
}