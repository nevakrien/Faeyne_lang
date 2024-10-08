pub struct ChunkedList<T,N> {
	data:[MaybeUninit<T>,N],
	next:Option<Box<ChunkedList<T,N>>>
}