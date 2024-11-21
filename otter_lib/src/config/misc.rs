pub mod switches {
    //! Boolean valued context configurations
    //! When set to true things related to the identifier are enabled.

    #[derive(Clone, Debug)]
    pub struct Switches {
        pub preprocessing: bool,
        pub restart: bool,
        pub subsumption: bool,
    }

    impl Default for Switches {
        fn default() -> Self {
            Switches {
                preprocessing: false,
                restart: true,
                subsumption: true,
            }
        }
    }
}
