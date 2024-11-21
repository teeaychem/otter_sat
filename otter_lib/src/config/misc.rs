pub mod switches {
    //! Boolean valued context configurations
    //! When set to true things related to the identifier are enabled.

    #[derive(Clone, Debug)]
    pub struct Switches {
        pub phase_saving: bool,
        pub preprocessing: bool,
        pub restart: bool,
        pub subsumption: bool,
    }

    impl Default for Switches {
        fn default() -> Self {
            Switches {
                phase_saving: true,
                preprocessing: false,
                restart: true,
                subsumption: true,
            }
        }
    }
}
