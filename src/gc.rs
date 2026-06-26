use crate::managed::*;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::collections::{hash_map::Entry, HashMap};
use std::ptr::NonNull;

pub struct Gc {
    heap: RefCell<Vec<Box<Allocation<dyn Manage>>>>,
    bytes_allocated: Cell<usize>,
    number_of_gcs: Cell<usize>,
    next_gc: Cell<usize>,
    string_cache: RefCell<HashMap<String, Managed<String>>>,
}

const GC_HEAP_GROW_FACTOR: usize = 2;

impl Gc {
    // Create a new garbage collector instance for objects
    pub fn new() -> Self{
        Gc {
            heap: RefCell::new(Vec::with_capacity(100)),
            bytes_allocated: Cell::new(0),
            number_of_gcs: Cell::new(0),
            next_gc: Cell::new(1024 * 1024),
            string_cache: RefCell::new(HashMap::new()),
        }
    }
    pub(crate) fn manage<T: Manage, C: GcTrace>
    (
        &self,
        data: T,
        context: &C
    ) -> Managed<T>
    {
        self.allocate(data, context)
    }

    pub(crate) fn manage_str<C: GcTrace>
    (
        &self,
        string: String,
        context: &C
    ) -> Managed<String>
    {
        let mut cache = self.string_cache.borrow_mut();
        match cache.entry(string) {
            Entry::Vacant(vacant) => {
                let managed = self.allocate(vacant.key().to_string(), context);
                *vacant.insert(managed)
            }
            Entry::Occupied(occupied) => *occupied.get(),
        }
    }

    pub fn clone_managed<T: Manage + Clone, C: GcTrace>
    (
        &self,
        managed: Managed<T>,
        context: &C,
    ) -> Managed<T>
    {
        let cloned =  (*managed).clone();
        self.allocate(cloned, context)
    }

    fn allocate<T: Manage, C: GcTrace>
    (
        &self,
        data: T,
        context: &C
    ) -> Managed<T>
    {
        let mut alloc = Box::new(Allocation::new(data));
        let ptr = unsafe { NonNull::new_unchecked(&mut *alloc) };

        // push onto heap
        let size = alloc.size();
        let allocated = self
            .bytes_allocated
            .replace(self.bytes_allocated.get() + size);
        self.heap.borrow_mut().push(alloc);

        let managed = Managed::from(ptr);

        #[cfg(feature = "debug_stress_gc")]
        {
            self.collect_garbage(context, managed);
        }

        if allocated + size > self.next_gc.get() {
            self.collect_garbage(context, managed);
        }

        #[cfg(feature = "debug_gc")]
        {
            println!(
                "{:p} allocate {} for {}",
                ptr.as_ptr(),
                size,
                unsafe { ptr.as_ref() }.debug()
            );
        }

        managed
    }

    fn collect_garbage<T: Manage, C: GcTrace>
    (
        &self,
        context: &C,
        last: Managed<T>
    ) 
    {
        
        #[cfg(feature = "debug_gc")]
        {
            let mut before = self.bytes_allocated.get();
            println!("-- gc begin");
        }

        let mut gray_stack = Vec::with_capacity(40);
        if self.mark(context, &mut gray_stack) {
            self.mark_obj(last.clone_dyn(), &mut gray_stack);
            self.trace(&mut gray_stack);

            self.sweep_string_cache();
            let remaining = self.sweep();

            self.bytes_allocated.set(remaining);

            self
                .next_gc
                .set(self.bytes_allocated.get() * GC_HEAP_GROW_FACTOR);
        }

        #[cfg(feature = "debug_gc")]
        {
            let now = self.bytes_allocated.get();
            println!("-- gc end");
            println!(
                "   collected {} bytes (from {} to {}) next at {}",
                before - now,
                before,
                now,
                self.next_gc.get()
            );
        }
    }

    fn mark<T: GcTrace>
    (
        &self,
        root: &T,
        gray_stack: &mut Vec<Managed<dyn Manage>>
    ) -> bool
    {
        root.trace(&mut |obj| self.mark_obj(obj, gray_stack))
    }

    fn trace
    (
        &self,
        gray_stack: &mut Vec<Managed<dyn Manage>>
    )
    {
        let mut obj_buffer: Vec<Managed<dyn Manage>> = Vec::with_capacity(60);

        while let Some(gray) = gray_stack.pop() {
            gray.trace(&mut |obj| obj_buffer.push(obj));

            obj_buffer.drain(..).for_each(|obj| {
                self.mark_obj(obj, gray_stack);
            })
        }
    }

    fn sweep(&self) -> usize {
        let mut remaining: usize = 0;

        self.heap.borrow_mut().retain(|obj| {
            let retain = (**obj).unmark();

            if retain {
                remaining += obj.size();
            }

            #[cfg(feature = "debug_gc")]
            {
                if !retain {
                    println!("{:p} free {}", &**obj, (**obj).debug());
                }
            }

            retain
        });

        remaining
    }

    fn sweep_string_cache
    (
        &self
    )
    {
        self.string_cache.borrow_mut().retain(|_, &mut string| {
            let retain = string.obj().marked();

            #[cfg(feature = "debug_gc")]
            {
                if !retain {
                    println!(
                        "{:p} remove string from cache {}",
                        &**string,
                        (*string).debug()
                    );
                }
            }

            retain
        });
    }

    fn mark_obj
    (
        &self,
        managed: Managed<dyn Manage>,
        gray_stack: &mut Vec<Managed<dyn Manage>>
    )
    {
        if managed.obj().mark() {
            return;
        }

        #[cfg(feature = "debug_gc")]
        {
            println!("{:p} mark {}", &*managed.obj(), managed.debug());
        }

        gray_stack.push(managed);
    }
}

pub struct NoGc();

impl fmt::Display for NoGc {
    fn fmt
    (
        &self, 
        _: &mut fmt::Formatter
    ) -> Result<(), fmt::Error>
    {
        unreachable!()
    }
}

impl GcTrace for NoGc {
    fn trace
    (
        &self, 
        _: &mut dyn FnMut(Managed<dyn Manage>)
    ) -> bool
    {
        false
    }
}

pub static NO_GC: NoGc = NoGc {};
