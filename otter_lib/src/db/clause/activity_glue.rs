use crate::config::{Activity, LBD};

/*
A basic struct two allow ordering on both clause activity and glue strength
It is unlikely this has much positive impact, but here it is…
 */

/// A combination of [clause activity](crate::config::Activity) and [lbd](LBD), used to sort clauses on an activity heap.
#[derive(Debug)]
pub struct ActivityLBD {
    /// The activity of a clause.
    pub activity: Activity,
    /// The lbd of a clause.
    pub lbd: LBD,
}

impl Default for ActivityLBD {
    fn default() -> Self {
        ActivityLBD {
            activity: 1.0,
            lbd: 0,
        }
    }
}

// `Revered` as max heap
use std::cmp::Ordering;
impl PartialOrd for ActivityLBD {
    /// [ActivityLBD] is ordered with precedence to lowest lbd and then least activity.
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

impl PartialEq for ActivityLBD {
    fn eq(&self, other: &Self) -> bool {
        self.lbd.eq(&other.lbd) && self.activity.eq(&other.activity)
    }
}
