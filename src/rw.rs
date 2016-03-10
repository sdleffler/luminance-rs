pub struct R;
pub struct W;
pub struct RW;

pub trait Readable {}
pub trait Writable {}

impl Readable for R {}
impl Readable for RW {}
impl Writable for W {}
impl Writable for RW {}
