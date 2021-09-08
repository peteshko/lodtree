//! Iterators over chunks

use crate::tree::*;
use crate::traits::*;

// helper to implement all iterators
macro_rules! impl_iterator {
	($name:ident, $type:ty, $len:ident, $get:ident) => {
		// define the struct
		/// Iterator over chunks in the tree.
		/// This struct was automatically generated by a macro
		pub struct $name<'a, C: Sized, L: LodVec> {
			tree: &'a Tree<C, L>,
			index: usize,
		}

		// and implement iterator for it
		impl<'a, C: Sized, L: LodVec> Iterator for $name<'a, C, L> {
			type Item = $type;

			fn next(&mut self) -> Option<Self::Item> {

				// if the item is too big, stop
				if self.index >= self.tree.$len() {
					None 
				} else {

					// otherwise, get the item
					let item = self.tree.$get(self.index);

					// increment the index
					self.index += 1;

					Some(item)
				}
			} 
		}
	};
}

// helper to implement all iterators, mutable version
// TODO
macro_rules! impl_iterator_mut {
	($name:ident, $type:ty, $len:ident, $get:ident) => {
		// define the struct
		/// Iterator over chunks in the tree.
		/// This struct was automatically generated by a macro
		pub struct $name<'a, C: Sized, L: LodVec> {
			tree: &'a mut Tree<C, L>,
			index: usize,
		}

		// and implement iterator for it
		impl<'a, C: Sized, L: LodVec> Iterator for $name<'a, C, L> {
			type Item = $type;

			fn next(&mut self) -> Option<Self::Item> {

				// if the item is too big, stop
				if self.index >= self.tree.$len() {
					None 
				} else {

					// otherwise, get the item
					let mut item = self.tree.$get(self.index);

					// increment the index
					self.index += 1;

					Some(item)
				}
			} 
		}
	};
}

impl_iterator!(ChunkIter, &'a C, get_num_chunks, get_chunk);
//impl_iterator_mut!(MutChunkIter, &'a mut C, get_num_chunks, get_chunk_mut);
impl_iterator!(ChunkToActivateIter, &'a C, get_num_chunks_to_activate, get_chunk_to_activate);
impl_iterator!(ChunkToDeactivateIter, &'a C, get_num_chunks_to_deactivate, get_chunk_to_deactivate);
impl_iterator!(ChunkToAddIter, &'a C, get_num_chunks_to_add, get_chunk_to_add);
impl_iterator!(PositionToAddIter, L, get_num_chunks_to_add, get_position_of_chunk_to_add);
impl_iterator!(PositionAndChunkToAddIter, (L, &'a C), get_num_chunks_to_add, get_position_and_chunk_to_add);
impl_iterator!(ChunkToRemoveIter, &'a C, get_num_chunks_to_remove, get_chunk_to_remove);

// TODO: iterator for all 

impl<'a, C, L> Tree<C, L>
where
    C: Sized,
    L: LodVec,
	Self: 'a,
{

	/// iterate over all chunks
	#[inline]
	pub fn iter_chunks(&self) -> ChunkIter<C, L> {
		ChunkIter { tree: self, index: 0 }
	}

	// iterate over all chunks, mutable

	/// iterate over all chunks to activate
	#[inline]
	pub fn iter_chunks_to_activate(&self) -> ChunkToActivateIter<C, L> {
		ChunkToActivateIter { tree: self, index: 0 }
	}

	// iterate over all chunks to activate, mut

	/// iterate over all chunks to deactivate
	#[inline]
	pub fn iter_chunks_to_deactivate(&self) -> ChunkToDeactivateIter<C, L> {
		ChunkToDeactivateIter { tree: self, index: 0 }
	}

	// iterate over all chunks to deactivate, mut

	/// iterate over all chunks to remove
	#[inline]
	pub fn iter_chunks_to_remove(&self) -> ChunkToRemoveIter<C, L> {
		ChunkToRemoveIter { tree: self, index: 0 }
	}

	// iterate over all chunks to remove, mut

	/// iterate over all chunks to add
	#[inline]
	pub fn iter_chunks_to_add(&self) -> ChunkToAddIter<C, L> {
		ChunkToAddIter { tree: self, index: 0 }
	}

	/// iterate over the positions of all chunks to add
	#[inline]
	pub fn iter_positions_of_chunks_to_add(&self) -> PositionToAddIter<C, L> {
		PositionToAddIter { tree: self, index: 0 }
	}

	/// iterate over the positions and all chunks to add
	#[inline]
	pub fn iter_positions_and_chunks_to_add(&self) -> PositionAndChunkToAddIter<C, L> {
		PositionAndChunkToAddIter { tree: self, index: 0 }
	}

	// iterate over the positions and all mutable chunks to add

	// iterate over all chunks to add, mut

	// iterate over all chunks that would be affected by an edit

	// iterate over all chunks affected by an edit

	// iterate over all chunks that would be affected by an edit, including the chunk if it's in the tree

}