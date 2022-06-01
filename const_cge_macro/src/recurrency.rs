use std::collections::HashMap;
use cge::{Network, gene::NeuronId};

#[derive(Copy, Clone)]
pub enum RecurrencyConstraint {
  Required,
  Forbidden,
  DontCare
}

pub fn identify_recurrence(network: &Network<f64>) -> HashMap<NeuronId, usize> {
  // literally just list all `Recurrent` genes that exist in the network. 
  // The ID field is the /source/ of the recurrence,
  // and our job is to number these neurons tightly (they will become indices in an array)
  let mut recurrence_table = HashMap::new();

  network
    .genome()
    .iter()
    .filter_map(|g| match g {
      cge::gene::Gene::RecurrentJumper(g) => Some(g.source_id()),
      _ => None
    })
    .enumerate()
    .for_each(|(index, id)| { recurrence_table.insert(id, index); });
  
  recurrence_table
}