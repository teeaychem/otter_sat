use crate::config::{ClauseActivity, GlueStrength};

/*
A basic struct two allow ordering on both clause activity and glue strength
It is unlikely this has much positive impact, but here it is…
 */

pub struct ActivityGlue {
    pub activity: ClauseActivity,
    pub lbd: GlueStrength,
}

impl Default for ActivityGlue {
    fn default() -> Self {
        ActivityGlue {
            activity: 0.0,
            lbd: 0,
        }
    }
}

// `Revered` as max heap
impl PartialOrd for ActivityGlue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let lbd_comparison = match self.lbd.cmp(&other.lbd) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => match self.activity.partial_cmp(&other.activity) {
                None => panic!("could not compare activity/lbd"),
                Some(comparison) => match comparison {
                    std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
                    std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
                    std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
                },
            },
        };
        Some(lbd_comparison)
    }
}

// impl PartialOrd for ActivityGlue {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         let lbd_comparison = match self.activity.partial_cmp(&other.activity) {
//             None => panic!("could not compare activity/lbd"),
//             Some(comparison) => match comparison {
//                 std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
//                 std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
//                 std::cmp::Ordering::Equal => match self.lbd.cmp(&other.lbd) {
//                     std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
//                     std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
//                     std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
//                 },
//             },
//         };
//         Some(lbd_comparison)
//     }
// }

impl PartialEq for ActivityGlue {
    fn eq(&self, other: &Self) -> bool {
        self.lbd.eq(&other.lbd) && self.activity.eq(&other.activity)
    }
}
