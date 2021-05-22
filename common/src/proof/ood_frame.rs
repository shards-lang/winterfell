// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{errors::ProofSerializationError, EvaluationFrame};
use math::{field::FieldElement, utils::read_elements_into_vec};
use serde::{Deserialize, Serialize};

// OUT-OF-DOMAIN EVALUATION FRAME
// ================================================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OodFrame {
    trace_at_z1: Vec<u8>,
    trace_at_z2: Vec<u8>,
    evaluations: Vec<u8>,
}

impl OodFrame {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Serializes the provided evaluation frame and a vector of out-of-domain constraint
    /// evaluations into vectors of bytes.
    pub fn new<E: FieldElement>(frame: EvaluationFrame<E>, evaluations: Vec<E>) -> Self {
        OodFrame {
            trace_at_z1: elements_to_canonical_bytes(&frame.current),
            trace_at_z2: elements_to_canonical_bytes(&frame.next),
            evaluations: elements_to_canonical_bytes(&evaluations),
        }
    }

    // UPDATERS
    // --------------------------------------------------------------------------------------------

    /// Updates this evaluation frame potion of this out-of-domain frame.
    pub fn set_evaluation_frame<E: FieldElement>(&mut self, frame: &EvaluationFrame<E>) {
        assert!(self.trace_at_z1.is_empty());
        assert!(self.trace_at_z2.is_empty());
        self.trace_at_z1 = elements_to_canonical_bytes(&frame.current);
        self.trace_at_z2 = elements_to_canonical_bytes(&frame.next);
    }

    pub fn set_constraint_evaluations<E: FieldElement>(&mut self, evaluations: &[E]) {
        assert!(self.evaluations.is_empty());
        self.evaluations = elements_to_canonical_bytes(evaluations);
    }

    // PARSER
    // --------------------------------------------------------------------------------------------
    /// Returns an evaluation frame and a vector of out-of-domain constraint evaluations parsed
    /// from the serialized byte vectors.
    pub fn parse<E: FieldElement>(
        self,
        trace_width: usize,
        num_evaluations: usize,
    ) -> Result<(EvaluationFrame<E>, Vec<E>), ProofSerializationError> {
        let current = read_elements_into_vec(&self.trace_at_z1)
            .map_err(|err| ProofSerializationError::FailedToParseOodFrame(err.to_string()))?;
        if current.len() != trace_width {
            return Err(ProofSerializationError::WrongNumberOfOodTraceElements(
                trace_width,
                current.len(),
            ));
        }

        let next = read_elements_into_vec(&self.trace_at_z2)
            .map_err(|err| ProofSerializationError::FailedToParseOodFrame(err.to_string()))?;
        if next.len() != trace_width {
            return Err(ProofSerializationError::WrongNumberOfOodTraceElements(
                trace_width,
                next.len(),
            ));
        }

        let evaluations = read_elements_into_vec(&self.evaluations)
            .map_err(|err| ProofSerializationError::FailedToParseOodFrame(err.to_string()))?;
        if evaluations.len() != num_evaluations {
            return Err(ProofSerializationError::WrongNumberOfOodEvaluationElements(
                num_evaluations,
                evaluations.len(),
            ));
        }

        Ok((EvaluationFrame { current, next }, evaluations))
    }

    // SERIALIZATION
    // --------------------------------------------------------------------------------------------

    /// Serializes this out-of-domain frame and appends the resulting bytes to the `target` vector.
    pub fn write_into(&self, target: &mut Vec<u8>) {
        // we do not append vector lengths because the lengths can be inferred from other proof
        // and AIR parameters
        target.extend_from_slice(&self.trace_at_z1);
        target.extend_from_slice(&self.trace_at_z2);
        target.extend_from_slice(&self.evaluations)
    }
}

impl Default for OodFrame {
    fn default() -> Self {
        OodFrame {
            trace_at_z1: Vec::new(),
            trace_at_z2: Vec::new(),
            evaluations: Vec::new(),
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// TODO: move to math crate and optimize?
fn elements_to_canonical_bytes<E: FieldElement>(elements: &[E]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(elements.len() * E::ELEMENT_BYTES);
    for element in elements {
        bytes.extend_from_slice(&element.to_canonical_bytes())
    }
    bytes
}
