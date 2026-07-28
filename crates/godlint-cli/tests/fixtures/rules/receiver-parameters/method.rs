struct Service;

impl Service {
    // Three declared parameters plus a receiver: within the limit.
    fn accepted(&self, one: u32, two: u32, three: u32) {
        work(one, two, three);
    }

    fn reported(&self, one: u32, two: u32, three: u32, four: u32) {
        work(one, two, three, four);
    }
}
