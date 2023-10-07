use itertools::Itertools;
use ndarray::{Array1, Array2};
use std::collections::BTreeSet;

pub fn calc_ranks(similarity_matrix: &Array2<f64>) -> Vec<f64> {
    let sentence_count = similarity_matrix.shape()[1];
    let init_w = 1.0 / sentence_count as f64;
    let mut result = Array1::from(vec![init_w; sentence_count]);
    let mut prev_result = result.clone();

    let damping_factor = 0.85;
    let initial_m =
        damping_factor * similarity_matrix + (1.0 - damping_factor) / sentence_count as f64;
    let threshold = 0.001;
    loop {
        result = initial_m.dot(&result);
        let delta = &result - &prev_result;
        if delta.iter().all(|d| d <= &threshold) {
            break;
        }
        prev_result = result.clone();
    }
    result.into_raw_vec()
}

pub fn similarity_matrix(sentences: &Vec<Vec<String>>) -> Array2<f64> {
    let len = sentences.len();
    let mut matrix = Array2::<f64>::zeros((len, len));
    // calc similarity
    for i in 0..(len - 1) {
        for j in (i + 1)..len {
            let s = sentence_similarity(&sentences[i], &sentences[j]);
            matrix[[i, j]] = s;
            matrix[[j, i]] = s;
        }
    }
    // calc sum per column
    let sum_column = matrix
        .columns()
        .into_iter()
        .map(|c| c.into_iter().sum::<f64>())
        .collect_vec();
    // normalize
    for i in 0..len {
        for j in 0..len {
            if i != j {
                matrix[[i, j]] = matrix[[i, j]] / sum_column[j];
            }
        }
    }
    matrix
}

fn get_sentence_vector(sentence: &Vec<String>, word_bag: &BTreeSet<&str>) -> Vec<usize> {
    let mut vector: Vec<usize> = vec![0; word_bag.len()];
    for w in sentence {
        let index = word_bag.iter().position(|x| x.eq(&w)).unwrap();
        vector[index] += 1;
    }
    vector
}

fn sentence_similarity(s1: &Vec<String>, s2: &Vec<String>) -> f64 {
    let bag = word_bag(s1, s2);
    let v1 = get_sentence_vector(s1, &bag);
    let v2 = get_sentence_vector(s2, &bag);
    cosine_distance(&v1, &v2)
}

fn word_bag<'a>(s1: &'a Vec<String>, s2: &'a Vec<String>) -> BTreeSet<&'a str> {
    let words = s1.iter().chain(s2.into_iter());
    let mut set = BTreeSet::new();
    for w in words {
        set.insert(w.as_str());
    }
    set
}

fn cosine_distance(vec1: &Vec<usize>, vec2: &Vec<usize>) -> f64 {
    let dot_product = dot_product(vec1, vec2);
    let m1 = magnitude(vec1);
    let m2 = magnitude(vec2);
    return 1.0 - (dot_product as f64 / (m1 * m2));
}

fn dot_product(vec1: &Vec<usize>, vec2: &Vec<usize>) -> usize {
    vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum()
}

fn magnitude(vec: &Vec<usize>) -> f64 {
    (vec.iter().map(|a| a * a).sum::<usize>() as f64).sqrt()
}
