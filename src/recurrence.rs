use std::collections::HashMap;
use cge::{Network, gene::GeneExtras};

pub fn identify_recurrence(network: &Network) -> HashMap<usize, usize> {
  // literally just list all `Recurrent` genes that exist in the network. 
  // The ID field is the /source/ of the recurrence,
  // and our job is to number these neurons tightly (they will become indices in an array)
  network.genome.iter()
    .filter(|g| matches!(g.variant, GeneExtras::Recurrent))
    .enumerate()
    .map(|(index, gene)| (gene.id, index))
    .collect::<HashMap<usize, usize>>()
}