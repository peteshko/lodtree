//! Contains the tree struct, which is used to hold all chunks

use crate::traits::*;

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::num::NonZeroU32;

// struct for keeping track of chunks
// keeps track of the parent and child indices
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct TreeNode {
    // children, these can't be the root (index 0), so we can use Some and Nonzero for slightly more compact memory
    // children are also contiguous, so we can assume that this to this + num children - 1 are all the children of this node
    pub(crate) children: Option<NonZeroU32>,

    // where the chunk for this node is stored
    pub(crate) chunk: u32, //TODO change this to Option<NonZeroU32> such that nodes with no chunks can be created
}

//TODO: a better treenode structure
// Tree node that encompasses 4/8 children at once. Each child has two pointers:
// children[i] will point to the TreeNode which encompasses its children
// chunk[i] will point to the data chunk.
// both pointers may be "None", indicating either no children, or no data
//#[derive(Clone, Debug)]
// pub(crate) struct TreeNode2<L: LodVec>  where [(); L::NUM_CHILDREN as usize]: {
//     // children, these can't be the root (index 0), so we can use Some and Nonzero for slightly more compact memory
//     // children are also contiguous, so we can assume that this to this + num children - 1 are all the children of this node
//     pub(crate) children: [Option<NonZeroU32>; L::NUM_CHILDREN as usize],
//
//     // where the chunk for this node is stored
//     pub(crate) chunk: [Option<NonZeroU32>; L::NUM_CHILDREN as usize],
// }

// utility struct for holding actual chunks and the node that owns them
#[derive(Clone, Debug)]
pub(crate) struct ChunkContainer<C: Sized, L: LodVec> {
    pub(crate) chunk: C,    // actual data inside the chunk
    pub(crate) index: u32,  // index of the node that holds this chunk
    pub(crate) position: L, // where the chunk is (as this can not be recovered from node tree)
}

/// holds a chunk to add and it's position
/// modifying the position won't have any effect on where the chunk is placed in the tree
/// however it will be different when retrieving chunks from the tree
#[derive(Clone, Debug)]
pub struct ToAddContainer<C: Sized, L: LodVec> {
    /// The chunk that's going to be added
    pub chunk: C,

    /// Position of the chunk to add
    pub position: L,
    /// Index of the parent node
    parent_node_index: u32,
}

// utility struct for holding chunks to remove
#[derive(Clone, Debug)]
struct ToRemoveContainer {
    chunk: u32,  // chunk index
    parent: u32, // parent index
}

/// holds a chunk that's going to be deleted and it's position
#[derive(Clone, Debug)]
pub struct ToDeleteContainer<C: Sized, L: LodVec> {
    /// The chunk that's going to be deleted
    pub chunk: C,

    /// Position of the chunk
    pub position: L,
}

// utility struct for holding chunks in the queue
#[derive(Clone, Debug)]
struct QueueContainer<L: LodVec> {
    node: u32,   // chunk index
    position: L, // and it's position
}

// Tree holding all chunks
// partially based on: https://stackoverflow.com/questions/41946007/efficient-and-well-explained-implementation-of-a-quadtree-for-2d-collision-det
// assumption here is that because of the fact that we need to keep inactive chunks in memory for later use, we can keep them together with the actual nodes.
#[derive(Clone, Debug)]
pub struct Tree<C: Sized, L: LodVec> {
    /// All chunks in the tree
    pub(crate) chunks: Vec<ChunkContainer<C, L>>,

    /// nodes in the Tree
    pub(crate) nodes: Vec<TreeNode>,

    /// list of free nodes in the Tree, to allocate new nodes into
    free_list: VecDeque<u32>,

    /// actual chunks to add during next update
    chunks_to_add: Vec<ToAddContainer<C, L>>,

    /// chunk indices to be removed, tuple of index, parent index
    chunks_to_remove: Vec<ToRemoveContainer>,

    /// indices of the chunks that need to be activated (i.e. the chunks that have just lost children)
    chunks_to_activate: Vec<u32>,

    /// indices of the chunks that need to be deactivated (i.e. chunks that have been subdivided in this iteration)
    chunks_to_deactivate: Vec<u32>,

    /// internal queue for processing, that way we won't need to reallocate it
    processing_queue: Vec<QueueContainer<L>>,

    /// cache size, determines the max amount of elements in the cache
    cache_size: usize,

    /// internal chunk cache
    chunk_cache: HashMap<L, C>,

    /// tracking queue, to see which chunks are oldest
    cache_queue: VecDeque<L>,

    /// chunks that are going to be permamently removed, due to not fitting in the cache anymore
    chunks_to_delete: Vec<ToDeleteContainer<C, L>>,
}

impl<C, L> Tree<C, L>
where
    C: Sized,
    L: LodVec,
{
    /// Gets an index in self.nodes vector from a position.
    /// If position is not pointing to a node, None is returned.
    fn get_node_index_from_position(&self, position: L) -> Option<usize> {
        // the current node
        let mut current = *self.nodes.get(0)?;

        // and position
        let mut current_position = L::root();

        // then loop
        loop {
            // if the current node is the one we are looking for, return
            if current_position == position {
                return Some(current.chunk as usize);
            }

            // if the current node does not have children, stop
            // this works according to clippy
            current.children?;

            // if not, go over the node children
            if let Some((index, found_position)) = (0..L::NUM_CHILDREN)
                .map(|i| (i, current_position.get_child(i)))
                .find(|(_, x)| x.contains_child_node(position))
            {
                // we found the position to go to
                current_position = found_position;

                // and the node is at the index of the child nodes + index
                current = self.nodes[(current.children.unwrap().get() + index) as usize];
            } else {
                // if no child got found that matched the item, return none
                return None;
            }
        }
    }

    /// Create a new, empty tree, with a cache of given size
    ///  Set cache to zero to disable it entirely (may speed up certain workloads by ~50%)
    pub fn new(cache_size: usize) -> Self {
        // make a new Tree
        // also allocate some room for nodes
        Self {
            chunks_to_add: Vec::new(),
            chunks_to_remove: Vec::new(),
            chunks_to_activate: Vec::new(),
            chunks_to_deactivate: Vec::new(),
            chunks: Vec::new(),
            nodes: Vec::new(),
            free_list: VecDeque::new(),
            processing_queue: Vec::new(),
            cache_size,
            chunk_cache: HashMap::with_capacity(cache_size),
            cache_queue: VecDeque::with_capacity(cache_size),
            chunks_to_delete: Vec::with_capacity(cache_size),
        }
    }

    /// create a tree with preallocated memory for chunks and nodes
    ///  Set cache to zero to disable it entirely (may speed up certain workloads by ~50%)
    pub fn with_capacity(capacity: usize, cache_size: usize) -> Self {
        // make a new Tree
        // also allocate some room for nodes
        Self {
            chunks_to_add: Vec::with_capacity(capacity),
            chunks_to_remove: Vec::with_capacity(capacity),
            chunks_to_activate: Vec::with_capacity(capacity),
            chunks_to_deactivate: Vec::with_capacity(capacity),
            chunks: Vec::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            free_list: VecDeque::with_capacity(capacity),
            processing_queue: Vec::with_capacity(capacity),
            cache_size,
            chunk_cache: HashMap::with_capacity(cache_size),
            cache_queue: VecDeque::with_capacity(cache_size),
            chunks_to_delete: Vec::with_capacity(cache_size),
        }
    }

    /// get the number of chunks in the tree
    #[inline]
    pub fn get_num_chunks(&self) -> usize {
        self.chunks.len()
    }

    /// get a chunk
    #[inline]
    pub fn get_chunk(&self, index: usize) -> &C {
        &self.chunks[index].chunk
    }

    /// get a chunk by position, or none if it's not in the tree
    #[inline]
    pub fn get_chunk_from_position(&self, position: L) -> Option<&C> {
        // get the index of the chunk
        let chunk_index = self.get_node_index_from_position(position)?;

        // and return the chunk
        Some(&self.chunks[chunk_index].chunk)
    }

    /// get a mutable chunk by position, or none if it's not in the tree
    #[inline]
    pub fn get_chunk_from_position_mut(&mut self, position: L) -> Option<&mut C> {
        // get the index of the chunk
        let chunk_index = self.get_node_index_from_position(position)?;

        // and return the chunk
        Some(&mut self.chunks[chunk_index].chunk)
    }

    /// get a chunk as mutable
    #[inline]
    pub fn get_chunk_mut(&mut self, index: usize) -> &mut C {
        &mut self.chunks[index].chunk
    }

    /// gets a mutable pointer to a chunk
    /// This casts get_chunk_mut to a pointer underneath the hood
    #[inline]
    pub fn get_chunk_pointer_mut(&mut self, index: usize) -> *mut C {
        self.get_chunk_mut(index)
    }

    /// get the position of a chunk
    #[inline]
    pub fn get_chunk_position(&self, index: usize) -> L {
        self.chunks[index].position
    }

    /// get the number of chunks pending activation
    #[inline]
    pub fn get_num_chunks_to_activate(&self) -> usize {
        self.chunks_to_activate.len()
    }

    /// get a chunk pending activation
    #[inline]
    pub fn get_chunk_to_activate(&self, index: usize) -> &C {
        &self.chunks[self.nodes[self.chunks_to_activate[index] as usize].chunk as usize].chunk
    }

    /// get a mutable chunk pending activation
    #[inline]
    pub fn get_chunk_to_activate_mut(&mut self, index: usize) -> &mut C {
        &mut self.chunks[self.nodes[self.chunks_to_activate[index] as usize].chunk as usize].chunk
    }

    /// gets a mutable pointer to a chunk that is pending activation
    /// This casts get_chunk_to_activate_mut to a pointer underneath the hood
    #[inline]
    pub fn get_chunk_to_activate_pointer_mut(&mut self, index: usize) -> *mut C {
        self.get_chunk_to_activate_mut(index)
    }

    /// get the position of a chunk pending activation
    #[inline]
    pub fn get_position_of_chunk_to_activate(&self, index: usize) -> L {
        self.chunks[self.nodes[self.chunks_to_activate[index] as usize].chunk as usize].position
    }

    /// get the number of chunks pending deactivation
    #[inline]
    pub fn get_num_chunks_to_deactivate(&self) -> usize {
        self.chunks_to_deactivate.len()
    }

    /// get a chunk pending deactivation
    #[inline]
    pub fn get_chunk_to_deactivate(&self, index: usize) -> &C {
        &self.chunks[self.nodes[self.chunks_to_deactivate[index] as usize].chunk as usize].chunk
    }

    /// get a mutable chunk pending deactivation
    #[inline]
    pub fn get_chunk_to_deactivate_mut(&mut self, index: usize) -> &mut C {
        &mut self.chunks[self.nodes[self.chunks_to_deactivate[index] as usize].chunk as usize].chunk
    }

    /// gets a mutable pointer to a chunk that is pending deactivation
    /// This casts get_chunk_to_deactivate_mut to a pointer underneath the hood
    #[inline]
    pub fn get_chunk_to_deactivate_pointer_mut(&mut self, index: usize) -> *mut C {
        self.get_chunk_to_deactivate_mut(index)
    }

    /// get the position of a chunk pending deactivation
    #[inline]
    pub fn get_position_of_chunk_to_deactivate(&self, index: usize) -> L {
        self.chunks[self.nodes[self.chunks_to_deactivate[index] as usize].chunk as usize].position
    }

    /// get the number of chunks pending removal
    #[inline]
    pub fn get_num_chunks_to_remove(&self) -> usize {
        self.chunks_to_remove.len()
    }

    /// get a chunk pending removal
    #[inline]
    pub fn get_chunk_to_remove(&self, index: usize) -> &C {
        &self.chunks[self.nodes[self.chunks_to_remove[index].chunk as usize].chunk as usize].chunk
    }

    /// get a mutable chunk pending removal
    #[inline]
    pub fn get_chunk_to_remove_mut(&mut self, index: usize) -> &mut C {
        &mut self.chunks[self.nodes[self.chunks_to_remove[index].chunk as usize].chunk as usize]
            .chunk
    }

    /// gets a mutable pointer to a chunk that is pending removal
    /// This casts get_chunk_to_remove_mut to a pointer underneath the hood
    #[inline]
    pub fn get_chunk_to_remove_pointer_mut(&mut self, index: usize) -> *mut C {
        self.get_chunk_to_remove_mut(index)
    }

    /// get the position of a chunk pending removal
    #[inline]
    pub fn get_position_of_chunk_to_remove(&self, index: usize) -> L {
        self.chunks[self.nodes[self.chunks_to_remove[index].chunk as usize].chunk as usize].position
    }

    /// get the number of chunks to be added
    #[inline]
    pub fn get_num_chunks_to_add(&self) -> usize {
        self.chunks_to_add.len()
    }

    /// get a chunk that's going to be added
    #[inline]
    pub fn get_chunk_to_add(&self, index: usize) -> &C {
        &self.chunks_to_add[index].chunk
    }

    /// get a mutable chunk that's going to be added
    #[inline]
    pub fn get_chunk_to_add_mut(&mut self, index: usize) -> &mut C {
        &mut self.chunks_to_add[index].chunk
    }

    /// gets a mutable pointer to a chunk that is pending to be added
    /// This casts get_chunk_to_add_mut to a pointer underneath the hood
    #[inline]
    pub fn get_chunk_to_add_pointer_mut(&mut self, index: usize) -> *mut C {
        self.get_chunk_to_add_mut(index)
    }

    /// get the position of a chunk that's going to be added
    #[inline]
    pub fn get_position_of_chunk_to_add(&self, index: usize) -> L {
        self.chunks_to_add[index].position
    }

    /// gets the positions and chunks to be added as a slice
    #[inline]
    pub fn get_chunks_to_add_slice(&self) -> &[ToAddContainer<C, L>] {
        &self.chunks_to_add[..]
    }

    /// gets the positions and chunks to be added as a mutable slice
    #[inline]
    pub fn get_chunks_to_add_slice_mut(&mut self) -> &mut [ToAddContainer<C, L>] {
        &mut self.chunks_to_add[..]
    }

    /// get the number of chunks to be delete
    #[inline]
    pub fn get_num_chunks_to_delete(&self) -> usize {
        self.chunks_to_delete.len()
    }

    /// get a chunk that's going to be delete
    #[inline]
    pub fn get_chunk_to_delete(&self, index: usize) -> &C {
        &self.chunks_to_delete[index].chunk
    }

    /// get a mutable chunk that's going to be delete
    #[inline]
    pub fn get_chunk_to_delete_mut(&mut self, index: usize) -> &mut C {
        &mut self.chunks_to_delete[index].chunk
    }

    /// gets a mutable pointer to a chunk that is pending deletion
    /// This casts get_chunk_to_delete_mut to a pointer underneath the hood
    #[inline]
    pub fn get_chunk_to_delete_pointer_mut(&mut self, index: usize) -> *mut C {
        self.get_chunk_to_delete_mut(index)
    }

    /// get the position of a chunk that's going to be delete
    #[inline]
    pub fn get_position_of_chunk_to_delete(&self, index: usize) -> L {
        self.chunks_to_delete[index].position
    }

    /// gets the positions and chunks to be delete as a slice
    #[inline]
    pub fn get_chunks_to_delete_slice(&self) -> &[ToDeleteContainer<C, L>] {
        &self.chunks_to_delete[..]
    }

    /// gets the positions and chunks to be delete as a mutable slice
    #[inline]
    pub fn get_chunks_to_delete_slice_mut(&mut self) -> &mut [ToDeleteContainer<C, L>] {
        &mut self.chunks_to_delete[..]
    }

    /// Adds chunks at and around specified locations.
    /// This operation will also add chunks at other locations around the target to fullfill the
    /// datastructure constraints (such that no partially filled nodes exist).
    pub fn prepare_insert(
        &mut self,
        targets: &[L],
        detail: u32,
        chunk_creator: &mut dyn FnMut(L) -> C,
    ) -> bool {
        //FIXME: this function currently will dry-run once for every update to make sure
        // there is nothing left to update. This is a waste of CPU time, especially for many targets

        // first, clear the previous arrays
        self.chunks_to_add.clear();
        self.chunks_to_remove.clear();
        self.chunks_to_activate.clear();
        self.chunks_to_deactivate.clear();

        // if we don't have a root, make one pending for creation
        if self.nodes.is_empty() {
            // chunk to add
            let chunk_to_add = self.get_chunk_from_cache(L::root(), chunk_creator);

            // we need to add the root as pending
            self.chunks_to_add.push(ToAddContainer {
                position: L::root(),
                chunk: chunk_to_add,
                parent_node_index:0,
            });

            // and an update is needed
            return true;
        }

        // clear the processing queue from any previous updates
        self.processing_queue.clear();

        // add the root node (always at 0, if there is no root we would have returned earlier) to the processing queue
        self.processing_queue.push(QueueContainer {
            position: L::root(),
            node: 0,
        });

        // then, traverse the tree, as long as something is inside the queue
        while let Some(QueueContainer {
            position: current_position,
            node: current_node_index,
        }) = self.processing_queue.pop()
        {
            // fetch the current node
            let current_node = self.nodes[current_node_index as usize];
            //dbg!(current_node_index, current_node);
            // if we can subdivide, and the current node does not have children, subdivide the current node
            if current_node.children.is_none() {
                //println!("adding children");
                // add children to be added
                for i in 0..L::NUM_CHILDREN {
                    // chunk to add
                    let chunk_to_add =
                        self.get_chunk_from_cache(current_position.get_child(i), chunk_creator);

                    // add the new chunk to be added
                    self.chunks_to_add.push(ToAddContainer {
                        position: current_position.get_child(i),
                        chunk: chunk_to_add,
                        parent_node_index:current_node_index,
                    });

                }

                // and add ourselves for deactivation
                self.chunks_to_deactivate.push(current_node_index);
            } else if let Some(index) = current_node.children {
                //println!("has children at {index:?}");
                // queue child nodes for processing
                for i in 0..L::NUM_CHILDREN {
                    // wether we can subdivide
                    let child_pos = current_position.get_child(i);
                    //dbg!(child_pos);
                    for t in targets {
                        if *t == child_pos {
                            //println!("Found match for target {t:?}");
                            self.chunks[(index.get() + i) as usize].chunk =
                                chunk_creator(child_pos);
                            continue;
                        }
                        if t.can_subdivide(child_pos, detail) {
                            self.processing_queue.push(QueueContainer {
                                position: child_pos,
                                node: index.get() + i,
                            });
                            break;
                        }
                    }
                }
            }
        }

        // and return whether an update needs to be done
        !self.chunks_to_add.is_empty()
    }

    // how it works:
    // each node contains a pointer to it's chunk data and first child
    // start from the root node, which is at 0
    // check if we can't subdivide, and if all children are leafs
    // if so, queue children for removal, and self for activation (child indices, chunk pointer)
    // if we can subdivide, and have no children, queue children for addition, and self for removal (child positions, chunk pointer)
    // if none of the above and have children, queue children for processing
    // processing queue is only the node position and node index

    // when removing nodes, do so in groups of num children, and use the free list
    // clear the free list once we only have one chunk (the root) active
    // swap remove chunks, and update the node that references them (nodes won't move due to free list)

    /// prepares the tree for an update, an update is an operation that
    /// adds chunks around specified locations (targets) while also erasing all other chunks.
    /// this fills the internal lists of what chunks need to be added or removed as appropriate.
    /// # Params
    /// * `targets` The target positions to generate the lod around (QuadVec and OctVec define the center position and max lod in depth for this)
    /// * `detail` The detail for these targets (QuadVec and OctVec define this as amount of chunks around this point)
    /// * `chunk_creator` function to create a new chunk from a given position
    /// returns whether any update is needed.
    pub fn prepare_update(
        &mut self,
        targets: &[L],
        detail: u32,
        chunk_creator: &mut dyn FnMut(L) -> C,
    ) -> bool {
        //FIXME: this function currently will dry-run once for every update to make sure
        // there is nothing left to update. This is a waste of CPU time, especially for many targets

        // first, clear the previous arrays
        self.chunks_to_add.clear();
        self.chunks_to_remove.clear();
        self.chunks_to_activate.clear();
        self.chunks_to_deactivate.clear();

        // if we don't have a root, make one pending for creation
        if self.nodes.is_empty() {
            // chunk to add
            let chunk_to_add = self.get_chunk_from_cache(L::root(), chunk_creator);

            // we need to add the root as pending
            self.chunks_to_add.push(ToAddContainer {
                position: L::root(),
                chunk: chunk_to_add,
                parent_node_index:0,
            });


            // and an update is needed
            return true;
        }

        // clear the processing queue from any previous updates
        self.processing_queue.clear();

        // add the root node (always at 0, if there is no root we would have returned earlier) to the processing queue
        self.processing_queue.push(QueueContainer {
            position: L::root(),
            node: 0,
        });

        // then, traverse the tree, as long as something is inside the queue
        while let Some(QueueContainer {
            position: current_position,
            node: current_node_index,
        }) = self.processing_queue.pop()
        {
            // fetch the current node
            let current_node = self.nodes[current_node_index as usize];

            // wether we can subdivide
            let can_subdivide = targets
                .iter()
                .any(|x| x.can_subdivide(current_position, detail));

            // if we can subdivide, and the current node does not have children, subdivide the current node
            if can_subdivide && current_node.children.is_none() {
                // add children to be added
                for i in 0..L::NUM_CHILDREN {
                    // chunk to add
                    let chunk_to_add =
                        self.get_chunk_from_cache(current_position.get_child(i), chunk_creator);

                    // add the new chunk to be added
                    self.chunks_to_add.push(ToAddContainer {
                        position: current_position.get_child(i),
                        chunk: chunk_to_add,
                        parent_node_index:current_node_index,
                    });

                }

                // and add ourselves for deactivation
                self.chunks_to_deactivate.push(current_node_index);
            } else if let Some(index) = current_node.children {
                // otherwise, if we cant subdivide and have children, remove our children
                if !can_subdivide
                    && !(0..L::NUM_CHILDREN)
                        .into_iter()
                        .any(|i| self.nodes[(i + index.get()) as usize].children.is_some())
                {
                    // first, queue ourselves for activation
                    self.chunks_to_activate.push(current_node_index);

                    for i in 0..L::NUM_CHILDREN {
                        // no need to do this in reverse, that way the last node removed will be added to the free list, which is also the first thing used by the adding logic
                        self.chunks_to_remove.push(ToRemoveContainer {
                            chunk: index.get() + i,
                            parent: current_node_index,
                        });
                    }
                } else {
                    // queue child nodes for processing if we didn't subdivide or clean up our children
                    for i in 0..L::NUM_CHILDREN {
                        self.processing_queue.push(QueueContainer {
                            position: current_position.get_child(i),
                            node: index.get() + i,
                        });
                    }
                }
            }
        }

        // and return wether an update needs to be done
        !self.chunks_to_add.is_empty() || !self.chunks_to_remove.is_empty()
    }

    /// Runs the update that's stored in the internal lists.
    /// This adds and removes chunks based on that, however this assumes that chunks in the to_activate and to_deactivate list were manually activated or deactivated.
    /// This also assumes that the chunks in to_add had proper initialization, as they are added to the tree.
    /// After this, it's needed to clean un nodes in the chunk_to_delete list and call the function complete_update(), in order to properly clear the cache
    pub fn do_update(&mut self) {
        // no need to do anything with chunks that needed to be (de)activated, as we assume that has been handled beforehand

        // first, get the iterator for chunks that will be added
        // this becomes useful later
        let mut chunks_to_add_iter = self.chunks_to_add.drain(..);

        // then, remove old chunks, or cache them
        // we'll drain the vector, as we don't need it anymore afterward
        for ToRemoveContainer {
            chunk: index,
            parent: parent_index,
        } in self.chunks_to_remove.drain(..)
        // but we do need to cache these
        {
            // remove the node from the tree
            self.nodes[parent_index as usize].children = None;
            self.free_list.push_back(index);

            // and remove the chunk
            let chunk_index = self.nodes[index as usize].chunk;

            // but not so fast, because if we can overwrite it with a new chunk, do so
            // that way we can avoid a copy later on, which might be expensive
            if let Some(ToAddContainer { position, chunk, parent_node_index:parent_index}) =
                chunks_to_add_iter.next()
            {
                // add the node
                let new_node_index = match self.free_list.pop_front() {
                    Some(x) => {
                        // reuse a free node
                        self.nodes[x as usize] = TreeNode {
                            children: None,
                            chunk: chunk_index,
                        };

                        // old chunk that was previously in the array
                        // we initialize it to the new chunk, then swap them
                        let mut old_chunk = ChunkContainer {
                            index: x,
                            chunk,
                            position,
                        };

                        std::mem::swap(&mut old_chunk, &mut self.chunks[chunk_index as usize]);

                        // old chunk shouldn't be mutable anymore
                        let old_chunk = old_chunk;

                        // now, we can try to add this chunk into the cache
                        // first, remove any extra nodes if they are in the cache
                        while self.chunk_cache.len() > self.cache_size {
                            if let Some(chunk_position) = self.cache_queue.pop_front() {
                                // check if the chunk is inside the map
                                if let Some(cached_chunk) = self.chunk_cache.remove(&chunk_position)
                                {
                                    // if it is, it's removed, so we need to push it to the chunks that are going to be deleted
                                    self.chunks_to_delete.push(ToDeleteContainer {
                                        position: chunk_position,
                                        chunk: cached_chunk,
                                    });
                                }
                            } else {
                                // just break, otherwise we'll be stuck in an infinite loop
                                break;
                            }
                        }
                        if self.cache_size > 0 {
                            // then assign this chunk into the cache
                            if let Some(cached_chunk) =
                                self.chunk_cache.insert(old_chunk.position, old_chunk.chunk)
                            {
                                // there might have been another cached chunk
                                self.chunks_to_delete.push(ToDeleteContainer {
                                    position: old_chunk.position,
                                    chunk: cached_chunk,
                                });
                            }

                            // and make sure it's tracked
                            self.cache_queue.push_back(old_chunk.position);
                        }
                        x
                    }
                    // This can't be reached due to us *always* adding a chunk to the free list before popping it
                    None => unsafe { std::hint::unreachable_unchecked() },
                };

                // correctly set the children of the parent node.
                // because the last node we come by in with ordered iteration is on num_children - 1, we need to set it as such].
                // node 0 is the root, so the last child it has will be on num_children.
                // then subtracting num_children - 1 from that gives us node 1, which is the first child of the root.
                if new_node_index >= L::NUM_CHILDREN {
                    // because we loop in order, and our nodes are contiguous, the first node of the children got added on index i - (num children - 1)
                    // so we need to adjust for that
                    self.nodes[parent_index as usize].children =
                        NonZeroU32::new(new_node_index - (L::NUM_CHILDREN - 1));
                }
            } else {
                // otherwise we do need to do a regular swap remove
                let old_chunk = self.chunks.swap_remove(chunk_index as usize);

                // now, we can try to add this chunk into the cache
                // first, remove any extra nodes if they are in the cache
                while self.chunk_cache.len() > self.cache_size {
                    if let Some(chunk_position) = self.cache_queue.pop_front() {
                        // check if the chunk is inside the map
                        if let Some(cached_chunk) = self.chunk_cache.remove(&chunk_position) {
                            // if it is, it's removed, so we need to push it to the chunks that are going to be deleted
                            self.chunks_to_delete.push(ToDeleteContainer {
                                position: chunk_position,
                                chunk: cached_chunk,
                            });
                        }
                    } else {
                        // just break, otherwise we'll be stuck in an infinite loop
                        break;
                    }
                }
                if self.cache_size > 0 {
                    //then assign this chunk into the cache
                    if let Some(cached_chunk) =
                        self.chunk_cache.insert(old_chunk.position, old_chunk.chunk)
                    {
                        // there might have been another cached chunk
                        self.chunks_to_delete.push(ToDeleteContainer {
                            position: old_chunk.position,
                            chunk: cached_chunk,
                        });
                    }
                    // and make sure it's tracked
                    self.cache_queue.push_back(old_chunk.position);
                }
            }

            // and properly set the chunk pointer of the node of the chunk we just moved, if any
            // if we removed the last chunk, no need to update anything
            if chunk_index < self.chunks.len() as u32 {
                self.nodes[self.chunks[chunk_index as usize].index as usize].chunk = chunk_index;
            }
        }

        // add new chunks
        // we'll drain the vector here as well, as we won't need it anymore afterward
        for ToAddContainer { position, chunk, parent_node_index:parent_index } in chunks_to_add_iter {
            // add the node
            let new_node_index = match self.free_list.pop_front() {
                Some(x) => {
                    // reuse a free node
                    self.nodes[x as usize] = TreeNode {
                        children: None,
                        chunk: self.chunks.len() as u32,
                    };
                    self.chunks.push(ChunkContainer {
                        index: x,
                        chunk,
                        position,
                    });
                    x
                }
                None => {
                    // otherwise, use a new index
                    self.nodes.push(TreeNode {
                        children: None,
                        chunk: self.chunks.len() as u32,
                    });
                    self.chunks.push(ChunkContainer {
                        index: self.nodes.len() as u32 - 1,
                        chunk,
                        position,
                    });
                    (self.nodes.len() - 1) as u32
                }
            };

            // correctly set the children of the parent node.
            // because the last node we come by in with ordered iteration is on num_children - 1, we need to set it as such].
            // node 0 is the root, so the last child it has will be on num_children.
            // then subtracting num_children - 1 from that gives us node 1, which is the first child of the root.
            if new_node_index >= L::NUM_CHILDREN {
                // because we loop in order, and our nodes are contiguous, the first node of the children got added on index i - (num children - 1)
                // so we need to adjust for that
                self.nodes[parent_index as usize].children =
                    NonZeroU32::new(new_node_index - (L::NUM_CHILDREN - 1));
            }
        }

        // if there's only chunk left, we know it's the root, so we can get rid of all free nodes and unused nodes
        if self.chunks.len() == 1 {
            self.free_list.clear();
            self.nodes.resize(
                1,
                TreeNode {
                    children: None,
                    chunk: 0,
                },
            );
        }

        // and clear all internal arrays, so if this method is accidentally called twice, no weird behavior would happen
        self.chunks_to_add.clear();
        self.chunks_to_remove.clear();
        self.chunks_to_activate.clear();
        self.chunks_to_deactivate.clear();
    }

    /// Completes the update by removing all chunks that can't be stored anymore permanently
    #[inline]
    pub fn complete_update(&mut self) {
        // just clear the chunks to be deleted
        self.chunks_to_delete.clear();
    }

    /// clears the tree, removing all chunks and internal lists and cache
    #[inline]
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.nodes.clear();
        self.free_list.clear();
        self.chunks_to_add.clear();
        self.chunks_to_remove.clear();
        self.chunks_to_activate.clear();
        self.chunks_to_deactivate.clear();
        self.chunks_to_delete.clear();
        self.processing_queue.clear();
        self.cache_queue.clear();
        self.chunk_cache.clear();
    }

    /// Shrinks all internal buffers to fit, reducing memory usage.
    /// Due to most of the intermediate processing buffers being cleared after an update is done, the next update might take longer due to needing to reallocate the memory.
    #[inline]
    pub fn shrink(&mut self) {
        // it should be possible to also shrink the nodes as well, and remove the free space, but this would be rather dificult to do
        // because we have groups of num_children
        // I'm leaving it out for now
        self.chunks.shrink_to_fit();
        self.nodes.shrink_to_fit();
        self.free_list.shrink_to_fit();
        self.chunks_to_add.shrink_to_fit();
        self.chunks_to_remove.shrink_to_fit();
        self.chunks_to_activate.shrink_to_fit();
        self.chunks_to_deactivate.shrink_to_fit();
        self.chunks_to_delete.shrink_to_fit();
        self.processing_queue.shrink_to_fit();
        self.cache_queue.shrink_to_fit();
    }

    /// resizes the current cache size
    /// actual resizing happens on the next update
    #[inline]
    pub fn set_cache_size(&mut self, cache_size: usize) {
        self.cache_size = cache_size;
    }

    // gets a chunk from the cache, otehrwise generates one from the given function
    #[inline]
    fn get_chunk_from_cache(&mut self, position: L, chunk_creator: &mut dyn FnMut(L) -> C) -> C {
        if self.cache_size > 0 {
            if let Some(chunk) = self.chunk_cache.remove(&position) {
                return chunk;
            }
        }
        chunk_creator(position)
    }
}

impl<C, L> Default for Tree<C, L>
where
    C: Sized,
    L: LodVec,
{
    /// creates a new, empty tree, with no cache
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::coords::*;

    struct TestChunk;

    #[test]
    fn update_tree() {
        // make a tree
        let mut tree = Tree::<TestChunk, QuadVec>::new(64);
        // as long as we need to update, do so
        for tgt in [QuadVec::new(1, 1, 2), QuadVec::new(2, 3, 2)] {
            dbg!(tgt);
            while tree.prepare_update(&[tgt], 0, &mut |_| TestChunk {}) {
                for c in tree.iter_chunks_to_activate_positions() {
                    println!("* {c:?}");
                }
                for c in tree.iter_chunks_to_deactivate_positions() {
                    println!("o {c:?}");
                }

                for c in tree.iter_chunks_to_remove_positions() {
                    println!("- {c:?}");
                }

                for c in tree.iter_chunks_to_add_positions() {
                    println!("+ {c:?}");
                }
                println!("updating...");
                // and actually update
                tree.do_update();
            }
        }
    }
    #[test]
    fn insert_into_tree() {
        // make a tree
        let mut tree = Tree::<TestChunk, QuadVec>::new(64);
        // as long as we need to update, do so
        for tgt in [
            QuadVec::new(1, 1, 1),
            QuadVec::new(0, 1, 1),
            QuadVec::new(2, 3, 2),
            QuadVec::new(2, 2, 2),
        ] {
            println!("====NEXT TARGET =====");
            dbg!(tgt);
            while tree.prepare_insert(&[tgt], 0, &mut |_| TestChunk {}) {
                for c in tree.iter_chunks_to_activate_positions() {
                    println!("* {c:?}");
                }
                for c in tree.iter_chunks_to_deactivate_positions() {
                    println!("o {c:?}");
                }

                for c in tree.iter_chunks_to_remove_positions() {
                    println!("- {c:?}");
                }

                for c in tree.iter_chunks_to_add_positions() {
                    println!("+ {c:?}");
                }
                println!("updating...");
                // and actually update
                tree.do_update();
            }
        }
    }

    #[test]
    pub fn things() {
        //
        // // and move the target
        // while tree.prepare_update(&[QuadVec::new(16, 8, 16)], 8, |_| TestChunk {}) {
        //     // and actually update
        //     tree.do_update();
        // }
        //
        // // get the resulting chunk from a search
        // let found_chunk = tree.get_chunk_from_position(QuadVec::new(16, 8, 16));
        //
        // // and find the resulting chunk
        // println!("{:?}", found_chunk.is_some());
        //
        // // and make the tree have no items
        // while tree.prepare_update(&[], 8, |_| TestChunk {}) {
        //     // and actually update
        //     tree.do_update();
        // }
        //
        // // and do the same for an octree
        // let mut tree = Tree::<TestChunk, OctVec>::new(64);
        //
        // // as long as we need to update, do so
        // while tree.prepare_update(&[OctVec::new(128, 128, 128, 32)], 8, |_| TestChunk {}) {
        //     // and actually update
        //     tree.do_update();
        // }
        //
        // // and move the target
        // while tree.prepare_update(&[OctVec::new(16, 8, 32, 16)], 8, |_| TestChunk {}) {
        //     // and actually update
        //     tree.do_update();
        // }
        //
        // // and make the tree have no items
        // while tree.prepare_update(&[], 8, |_| TestChunk {}) {
        //     // and actually update
        //     tree.do_update();
        // }
    }
    #[test]
    pub fn alignment() {
        assert_eq!(std::mem::size_of::<TreeNode>(), 8);
    }
}
