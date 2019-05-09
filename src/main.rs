use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::Instant;

use rand::Rng;
use num_cpus;

fn main() {
    let now = Instant::now();
    let vec_length = 100_000;
    let vec_max = 10_000;
    let array_count = 100;

    for i in 0..array_count {
        let mut random_vec = get_random_vec( vec_length, vec_max );
        par_merge_sort( &mut random_vec );
        //println!( "{:?}", random_vec );

        //for j in 0..random_vec.len() {
            //if j > 0 {
                //assert!( random_vec[j] >= random_vec[j - 1] );
            //}
        //}

        //println!( "{}", i );
    }

    println!( "average parallel: {}", now.elapsed().as_millis() / array_count );

    let now = Instant::now();

    for i in 0..array_count {
        let mut random_vec = get_random_vec( vec_length, vec_max );
        merge_sort( &mut random_vec );
        //println!( "{:?}", random_vec );

        //for i in 0..random_vec.len() {
            //if i > 0 {
                //assert!( random_vec[i] >= random_vec[i - 1] );
            //}
        //}

        //println!( "{}", i );
    }

    println!( "average sequential: {}", now.elapsed().as_millis() / array_count );
}

fn par_merge_sort( a: &mut Vec<i32> ) {
    par_split( 0, a.len() as i32, a, ( num_cpus::get() as i32 ) / 2 - 1 );
    //par_split( &mut b, 0, a.len() as i32, a, 1 );
}

fn merge_sort( a: &mut Vec<i32> ) {
    let mut b = a.clone();

    split( &mut b, 0, a.len() as i32, a );
}

fn par_split<'a>( begin: i32, end: i32, a: &'a mut Vec<i32>, depth: i32 ) {
    if ( end - begin ) < 2 {
        return;
    }

    let middle = ( end + begin ) / 2;

    let arc_one = Arc::new( Mutex::new( ( a.clone(), begin, middle, end, depth ) ) );
    let arc_two = Arc::clone( &arc_one );

    let ( tx_one, rx_one ) = mpsc::channel();
    let ( tx_two, rx_two ) = mpsc::channel();

    thread::spawn( move || {
        let touple = arc_one.lock().unwrap();
        let mut a = touple.0.clone();
        let mut b = a.clone();
        let begin = touple.1;
        let middle = touple.2;
        let depth = touple.4;

        drop( touple );

        if depth <= 1 {
            split( &mut a, begin, middle, &mut b );
        } else {
            par_split( begin, middle, &mut b, depth - 1 );
        }

        tx_one.send( b ).unwrap();
    } );

    thread::spawn( move || {
        let touple = arc_two.lock().unwrap();
        let mut a = touple.0.clone();
        let mut b = a.clone();
        let middle = touple.2;
        let end = touple.3;
        let depth = touple.4;

        drop( touple );

        if depth <= 1 {
            split( &mut a, middle, end, &mut b );
        } else {
            par_split( middle, end, &mut b, depth - 1 );
        }

        tx_two.send( b ).unwrap();
    } );

    let b_one = rx_one.recv().unwrap();
    let b_two = rx_two.recv().unwrap();

    par_merge( b_one, b_two, begin as usize, middle as usize, end as usize, a );
}

fn split( b: &mut Vec<i32>, begin: i32, end: i32, a: &mut Vec<i32> ) {
    if ( end - begin ) < 2 {
        return;
    }

    let middle = ( end + begin ) / 2;

    split( a, begin, middle, b );
    split( a, middle, end, b );

    merge( b, begin as usize, middle as usize, end as usize, a );
}

fn merge( a: &mut Vec<i32>, begin: usize, middle: usize, end: usize, b: &mut Vec<i32> ) {
    let mut i = begin;
    let mut j = middle;

    for k in begin..end {
        if i < middle && ( j >= end || a[i] <= a[j] ) {
            b[k] = a[i];
            i += 1;
        } else {
            b[k] = a[j];
            j += 1;
        }
    }
}

fn par_merge( a_one: Vec<i32>, a_two: Vec<i32>, begin: usize, middle: usize, end: usize, b: &mut Vec<i32> ) {
    let mut i = begin;
    let mut j = middle;

    for k in begin..end {
        if i < middle && ( j >= end || a_one[i] <= a_two[j] ) {
            b[k] = a_one[i];
            i += 1;
        } else {
            b[k] = a_two[j];
            j += 1;
        }
    }
}

fn get_random_vec( len: i32, max: i32 ) -> Vec<i32> {
    let mut vec: Vec<i32> = Vec::with_capacity( len as usize );

    for _ in 0..len {
        vec.push( rand::thread_rng().gen_range( 0, max ) );
    }

    return vec;
}
