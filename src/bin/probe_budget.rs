use bpp::exact::*;
use bpp::adversarial::*;
fn main(){
    for n in [30usize,40,50] {
        let l = three_partition_hard(n);
        match optimal_bins_budget(&l, 600_000_000) { // 600 s = 10 min
            Some(o) => println!("RESULTADO pior_3particao n={n:3} OPT={o}"),
            None    => println!("RESULTADO pior_3particao n={n:3} NAO CONVERGE em 10min"),
        }
    }
}
