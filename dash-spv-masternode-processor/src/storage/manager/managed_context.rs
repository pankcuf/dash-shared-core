#[derive(Clone, Debug)]
pub struct ManagedContext {
    // pub pool: Pool<ConnectionManager<SqliteConnection>>,
    // pub chain: &'static ChainEntity,
    // prepared_for_save: HashMap<>
}

impl ManagedContext {
    pub const fn new_const_default() -> Self {
        Self { /*pool: get_connection_pool()*/ }
    }
}

impl Default for ManagedContext {
    fn default() -> Self {
        Self { /*pool: get_connection_pool()*/ }
    }
}

impl<'a> Default for &'a ManagedContext {
    fn default() -> &'a ManagedContext {
        static VALUE: ManagedContext = ManagedContext {};
        &VALUE
    }
}
