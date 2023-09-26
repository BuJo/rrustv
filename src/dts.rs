const DEVICE_TREE: &str = r#"
/dts-v1/;

/ {
	#address-cells = <1>;
	#size-cells = <1>;
	model = "BuJo,rriscv";
	aliases {
	};

	chosen {
	};

	firmware {
	};

    cpus {
        #address-cells = <1>;
        #size-cells = <0>;
        cpu@0 {
            device_type = "cpu";
            reg = <0>;
            status = "okay";
            compatible = "riscv";
            riscv,isa = "rv32i";
            clock-frequency = <0>;
            interrupt-controller {
                #interrupt-cells = <1>;
                interrupt-controller;
                compatible = "riscv,cpu-intc";
            }
        }
    }

    memory@0 {
		device_type = "memory";
		reg = <0x00000000 0x00002000>;
	};

	soc {
		#address-cells = <2>;
		#size-cells = <2>;
		compatible = "BuJo,rriscv-soc", "simple-bus";
		ranges;
		refclk: refclk {
			#clock-cells = <0>;
			compatible = "fixed-clock";
			clock-frequency = <1000>;
			clock-output-names = "xtal";
		};

		rom@1000 {
			compatible = "BuJo,bootrom";
			reg = <0x0 0x2000>;
			reg-names = "mem";
		};
    };
"#;
