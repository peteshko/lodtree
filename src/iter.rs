//! Iterators over chunks

use crate::traits::*;
use crate::tree::*;

// implements all iterators for the given functions
// this allows quickly and easily set them up for all chunks
// TODO: doc comments: In particular, it's the $(#[$($attrss:tt)*])* pattern to match attributes, and the $(#[$($attrss)*])* expression to emit them that you want.
macro_rules! impl_all_iterators {
    (
		$name:ident,
		$name_mut:ident,
		$name_pos:ident,
		$name_chunk_and_pos:ident,
		$name_chunk_and_pos_mut:ident,
		$len:ident,
		$get:ident,
		$get_mut:ident,
		$get_pos:ident,
		$func_name:ident,
		$func_name_mut:ident,
		$func_name_pos:ident,
		$func_name_chunk_and_pos:ident,
		$func_name_chunk_and_pos_mut:ident,
	) => {
        // define the struct
        /// Iterator for chunks, auto generated
        pub struct $name<'a, C: Sized, L: LodVec> {
            tree: &'a Tree<C, L>,
            index: usize,
        }

		/// Iterator for mutable chunks, auto generated
        pub struct $name_mut<'a, C: Sized, L: LodVec> {
            tree: &'a mut Tree<C, L>,
            index: usize,
        }

		/// Iterator for chunk positions, auto generated
        pub struct $name_pos<'a, C: Sized, L: LodVec> {
            tree: &'a Tree<C, L>,
            index: usize,
        }

		/// Iterator for chunks ans their positions, auto generated
        pub struct $name_chunk_and_pos<'a, C: Sized, L: LodVec> {
            tree: &'a Tree<C, L>,
            index: usize,
        }

		/// Iterator for mutable chunks and their positions, auto generated
        pub struct $name_chunk_and_pos_mut<'a, C: Sized, L: LodVec> {
            tree: &'a mut Tree<C, L>,
            index: usize,
        }

        // and implement iterator for it
        impl<'a, C: Sized, L: LodVec> Iterator for $name<'a, C, L> {
            type Item = &'a C;

			#[inline]
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

        impl<'a, C: Sized, L: LodVec> Iterator for $name_mut<'a, C, L> {
            type Item = &'a mut C;

			#[inline]
            fn next(&mut self) -> Option<Self::Item> {
                // if the item is too big, stop
                if self.index >= self.tree.$len() {
                    None
                } else {
                    // otherwise, get the item
                    let item = unsafe { self.tree.$get_mut(self.index).as_mut()? };

                    // increment the index
                    self.index += 1;

                    Some(item)
                }
            }
        }

        impl<'a, C: Sized, L: LodVec> Iterator for $name_pos<'a, C, L> {
            type Item = L;

			#[inline]
            fn next(&mut self) -> Option<Self::Item> {
                // if the item is too big, stop
                if self.index >= self.tree.$len() {
                    None
                } else {
                    // otherwise, get the item
                    let item = self.tree.$get_pos(self.index);

                    // increment the index
                    self.index += 1;

                    Some(item)
                }
            }
        }

        impl<'a, C: Sized, L: LodVec> Iterator for $name_chunk_and_pos<'a, C, L> {
            type Item = (&'a C, L);

			#[inline]
            fn next(&mut self) -> Option<Self::Item> {
                // if the item is too big, stop
                if self.index >= self.tree.$len() {
                    None
                } else {
                    // otherwise, get the item
                    let item = (self.tree.$get(self.index), self.tree.$get_pos(self.index));

                    // increment the index
                    self.index += 1;

                    Some(item)
                }
            }
        }

        impl<'a, C: Sized, L: LodVec> Iterator for $name_chunk_and_pos_mut<'a, C, L> {
            type Item = (&'a mut C, L);

			#[inline]
            fn next(&mut self) -> Option<Self::Item> {
                // if the item is too big, stop
                if self.index >= self.tree.$len() {
                    None
                } else {
                    // otherwise, get the item
                    let item = (
                        unsafe { self.tree.$get_mut(self.index).as_mut()? },
                        self.tree.$get_pos(self.index),
                    );

                    // increment the index
                    self.index += 1;

                    Some(item)
                }
            }
        }

        // exact size as well
        impl<'a, C: Sized, L: LodVec> ExactSizeIterator for $name<'a, C, L> {
			#[inline]
            fn len(&self) -> usize {
                self.tree.$len()
            }
        }

        impl<'a, C: Sized, L: LodVec> ExactSizeIterator for $name_mut<'a, C, L> {
			#[inline]
            fn len(&self) -> usize {
                self.tree.$len()
            }
        }

        impl<'a, C: Sized, L: LodVec> ExactSizeIterator for $name_pos<'a, C, L> {
			#[inline]
            fn len(&self) -> usize {
                self.tree.$len()
            }
        }

        impl<'a, C: Sized, L: LodVec> ExactSizeIterator for $name_chunk_and_pos<'a, C, L> {
			#[inline]
            fn len(&self) -> usize {
                self.tree.$len()
            }
        }

        impl<'a, C: Sized, L: LodVec> ExactSizeIterator for $name_chunk_and_pos_mut<'a, C, L> {
			#[inline]
            fn len(&self) -> usize {
                self.tree.$len()
            }
        }

        // fused, because it will always return none when done
        impl<'a, C: Sized, L: LodVec> std::iter::FusedIterator for $name<'a, C, L> {}
        impl<'a, C: Sized, L: LodVec> std::iter::FusedIterator for $name_mut<'a, C, L> {}
        impl<'a, C: Sized, L: LodVec> std::iter::FusedIterator for $name_pos<'a, C, L> {}
        impl<'a, C: Sized, L: LodVec> std::iter::FusedIterator for $name_chunk_and_pos<'a, C, L> {}
        impl<'a, C: Sized, L: LodVec> std::iter::FusedIterator
            for $name_chunk_and_pos_mut<'a, C, L>
        {
        }

        // and implement all of them for the tree
        impl<'a, C, L> Tree<C, L>
        where
            C: Sized,
            L: LodVec,
            Self: 'a,
        {
			#[inline]
			pub fn $func_name(&mut self) -> $name<C, L> {
				$name {
					tree: self,
					index: 0,
				}
			}

			#[inline]
			pub fn $func_name_mut(&mut self) -> $name_mut<C, L> {
				$name_mut {
					tree: self,
					index: 0,
				}
			}

			#[inline]
			pub fn $func_name_pos(&mut self) -> $name_pos<C, L> {
				$name_pos {
					tree: self,
					index: 0,
				}
			}

			#[inline]
			pub fn $func_name_chunk_and_pos(&mut self) -> $name_chunk_and_pos<C, L> {
				$name_chunk_and_pos {
					tree: self,
					index: 0,
				}
			}

			#[inline]
			pub fn $func_name_chunk_and_pos_mut(&mut self) -> $name_chunk_and_pos_mut<C, L> {
				$name_chunk_and_pos_mut {
					tree: self,
					index: 0,
				}
			}
        }
    };
}

// chunks
impl_all_iterators!(
    ChunkIter,
    ChunkIterMut,
    PositionIter,
    ChunkAndPositionIter,
    ChunkAndPositionIterMut,
    get_num_chunks,
    get_chunk,
    get_chunk_pointer_mut,
    get_chunk_position,
    iter_chunks,
    iter_chunks_mut,
    iter_chunk_positions,
    iter_chunks_and_positions,
    iter_chunks_and_positions_mut,
);

// to activate
impl_all_iterators!(
    ChunkToActivateIter,
    ChunkToActivateIterMut,
    PositionToActivateIter,
    ChunkAndPositionToActivateIter,
    ChunkAndPositionIterToActivateMut,
    get_num_chunks_to_activate,
    get_chunk_to_activate,
    get_chunk_to_activate_pointer_mut,
    get_position_of_chunk_to_activate,
    iter_chunks_to_activate,
    iter_chunks_to_activate_mut,
    iter_chunk_to_activate_positions,
    iter_chunks_and_positions_to_activate,
    iter_chunks_and_positions_to_activate_mut,
);

// to deactivate
impl_all_iterators!(
    ChunkToDeactivateIter,
    ChunkToDeactivateIterMut,
    PositionToDeactivateIter,
    ChunkAndPositionToDeactivateIter,
    ChunkAndPositionIterToDeactivateMut,
    get_num_chunks_to_deactivate,
    get_chunk_to_deactivate,
    get_chunk_to_deactivate_pointer_mut,
    get_position_of_chunk_to_deactivate,
    iter_chunks_to_deactivate,
    iter_chunks_to_deactivate_mut,
    iter_chunk_to_deactivate_positions,
    iter_chunks_and_positions_to_deactivate,
    iter_chunks_and_positions_to_deactivate_mut,
);

// to add
impl_all_iterators!(
    ChunkToAddIter,
    ChunkToAddIterMut,
    PositionToAddIter,
    ChunkAndPositionToAddIter,
    ChunkAndPositionIterToAddMut,
    get_num_chunks_to_add,
    get_chunk_to_add,
    get_chunk_to_add_pointer_mut,
    get_position_of_chunk_to_add,
    iter_chunks_to_add,
    iter_chunks_to_add_mut,
    iter_chunk_to_add_positions,
    iter_chunks_and_positions_to_add,
    iter_chunks_and_positions_to_add_mut,
);

// to remove
impl_all_iterators!(
    ChunkToRemoveIter,
    ChunkTorRmoveIterMut,
    PositionToRemoveIter,
    ChunkAndPositionToRemoveIter,
    ChunkAndPositionIterToRemoveMut,
    get_num_chunks_to_remove,
    get_chunk_to_remove,
    get_chunk_to_remove_pointer_mut,
    get_position_of_chunk_to_remove,
    iter_chunks_to_remove,
    iter_chunks_to_remove_mut,
    iter_chunk_to_remove_positions,
    iter_chunks_and_positions_to_remove,
    iter_chunks_and_positions_to_remove_mut,
);

// to delete
impl_all_iterators!(
    ChunkToDeleteIter,
    ChunkToDeleteIterMut,
    PositionToDeleteIter,
    ChunkAndPositionToDeleteIter,
    ChunkAndPositionIterToDeleteMut,
    get_num_chunks_to_delete,
    get_chunk_to_delete,
    get_chunk_to_delete_pointer_mut,
    get_position_of_chunk_to_delete,
    iter_chunks_to_delete,
    iter_chunks_to_delete_mut,
    iter_chunk_to_delete_positions,
    iter_chunks_and_positions_to_delete,
    iter_chunks_and_positions_to_delete_mut,
);

// iterator for all chunks that are inside given bounds
pub struct ChunksInBoundIter<L: LodVec> {
    // internal stack for which chunks are next
    stack: Vec<L>,

    // and maximum depth to go to
    max_depth: u64,

    // and the min of the bound
    bound_min: L,

    // and max of the bound
    bound_max: L,
}

// TODO: doesn't seem to work, due to is_inside_bounds
impl<L: LodVec> Iterator for ChunksInBoundIter<L> {
    type Item = L;

	#[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.stack.pop() {
            // go over all child nodes
            for i in 0..L::num_children() {
                let position = current.get_child(i);

                // if they are in bounds, and the correct depth, add them to the stack
                if position.is_inside_bounds(self.bound_min, self.bound_max, self.max_depth) {
                    self.stack.push(position);
                }
            }
            // and return this item from the stack
            Some(current)
        } else {
            None
        }
    }
}

impl<'a, C, L> Tree<C, L>
where
    C: Sized,
    L: LodVec,
    Self: 'a,
{
    // iterate over all chunks that would be affected by an edit inside a certain bound
    #[inline]
    pub fn iter_all_chunks_in_bounds(
        bound_min: L,
        bound_max: L,
        max_depth: u64,
    ) -> ChunksInBoundIter<L> {
        ChunksInBoundIter {
            stack: vec![L::root()],
            max_depth,
            bound_min,
            bound_max,
        }
    }

    // iterate over all chunks that would be affected by an edit, including the chunk if it's in the tree
	// might need to be implemented inside tree.rs
}

#[cfg(test)]
mod tests {

	use super::*;
	use crate::coords::*;

	#[test]
	fn test_bounds() {

		struct C;

		for pos in Tree::<C, QuadVec>::iter_all_chunks_in_bounds(QuadVec::new(1, 1, 4), QuadVec::new(7, 7, 4), 8) {

			println!("{:?}", pos);

		}
	}
}
