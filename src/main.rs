use rand::thread_rng;
use rand::Rng;
use Sumcheck::MultiVarPolynomial;

fn g_j(poly : MultiVarPolynomial, num_remaining_vars : usize, values: Vec<(usize, i32)>) -> MultiVarPolynomial {
    let mut c = if values.len() > 0 { poly.partial_eval( values) } else { poly };

    while c.num_vars > num_remaining_vars {
        c = c.bool_sum();
    }
    c
}

fn main() {
    // Read a polynomial from user input
    let polynomial = MultiVarPolynomial::read_from_input();
    println!("Parsed polynomial: {:?}", polynomial);
    
    //Setup the random number generator
    let mut rng = thread_rng();
    let mut values = vec![];

    //Prover calculates C and sends to verifier
    let c = g_j(polynomial.clone(), 0, values.clone());
    println!("C is: {:?}", c);

    let mut g_prev = c;

    for num_var in 1..polynomial.num_vars+1 {
        //Verifier sends random element of field F to prover
        let r;
        if num_var > 1 {
            r = rng.gen_range(0..polynomial.modulus);
            values.push((num_var-2,r));
            println!("r_{:?} is: {:?}", num_var-1, r);
        }
        else {
            r = 0;
        }
    
        //Prover calculates g_j(X_j) and sends to verifier
        let g = g_j(polynomial.clone(), 1, values.clone());
        println!("g_{num_var} is: {:?}", g);

        //Verifier checks g_j is a polynomial in 1 var, rejecting if not
         if g.num_vars != 1 {
            println!("Proof rejected!");
        }

         //Verifier checks degree g_j(X_j) <= deg_j(g), rejecting if not
        if g.degree_in_var(0) > polynomial.degree_in_var(num_var-1)  {
            println!("Proof rejected!");
        }

        //Verifier checks g_{j-1}(r_{j-1}) = g_j(0) + g_j(1), rejecting if not
        if  g_prev.partial_eval(if num_var > 1 {vec![(0, r)]} else {vec![]}) != g.bool_sum() {
            println!("Proof rejected!");
        }

        g_prev = g;
    }

    //Finally verifier picks last element
    let r = rng.gen_range(0..polynomial.modulus);
    values.push((polynomial.num_vars-1,r));
    println!("r_{:?} is: {:?}", polynomial.num_vars, r);

    //Verifier checks g(r_1, ..., r_n) = g_n(r_n), rejecting if not
    if polynomial.partial_eval(values) != g_prev.partial_eval(vec![(0,r)]) {
        println!("Proof rejected!");
    }

    println!("Proof accepted!")

}



#[cfg(test)]
mod tests {
    use super::*;

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
}