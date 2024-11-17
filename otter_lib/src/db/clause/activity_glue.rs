use crate::config::{Activity, GlueStrength};

/*
A basic struct two allow ordering on both clause activity and glue strength
It is unlikely this has much positive impact, but here it is…
 */

#[derive(Debug)]
pub(super) struct ActivityGlue {
    pub activity: Activity,
    pub lbd: GlueStrength,
}

impl Default for ActivityGlue {
    fn default() -> Self {
        ActivityGlue {
            activity: 1.0,
            lbd: 0,
        }
    }
}

// `Revered` as max heap
use std::cmp::Ordering;
impl PartialOrd for ActivityGlue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let lbd_comparison = match self.lbd.cmp(&other.lbd) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => {
                match self.activity.partial_cmp(&other.activity) {
                    None => Ordering::Equal, // TODO: consider
                    Some(comparison) => match comparison {
                        Ordering::Less => Ordering::Greater,
                        Ordering::Greater => Ordering::Less,
                        Ordering::Equal => Ordering::Equal,
                    },
                }
            }
        };
        Some(lbd_comparison)
    }
}

impl PartialEq for ActivityGlue {
    fn eq(&self, other: &Self) -> bool {
        self.lbd.eq(&other.lbd) && self.activity.eq(&other.activity)
    }
}
