import logging
import os
import shutil

import riscof.utils as utils
from riscof.pluginTemplate import pluginTemplate

logger = logging.getLogger()


class makeUtil(utils.makeUtil):
    def add_filetarget(self, command, output, input):
        with open(self.makefilePath, "a") as makefile:
            makefile.write("\n\n" + output + " : " + input + " \n\t" + command.replace("\n", "\n\t"))

    def add_phonytarget(self, input, tname=""):
        if tname == "":
            tname = "TARGET" + str(len(self.targets))
        with open(self.makefilePath, "a") as makefile:
            makefile.write("\n\n.PHONY : " + tname + "\n" + tname + " : " + input + "\n")
            self.targets.append(tname)


class sail_cSim(pluginTemplate):
    __model__ = "sail_c_simulator"
    __version__ = "0.5.0"

    def __init__(self, *args, **kwargs):
        sclass = super().__init__(*args, **kwargs)

        config = kwargs.get('config')
        if config is None:
            logger.error("Config node for sail_cSim missing.")
            raise SystemExit(1)
        self.num_jobs = str(config['jobs'] if 'jobs' in config else 1)
        if 'target_run' in config and config['target_run'] == '0':
            self.target_run = False
        else:
            self.target_run = True
        self.pluginpath = os.path.abspath(config['pluginpath'])
        self.sail_exe = {'32': os.path.join(config['PATH'] if 'PATH' in config else "", "riscv_sim_RV32"),
                         '64': os.path.join(config['PATH'] if 'PATH' in config else "", "riscv_sim_RV64")}
        self.isa_spec = os.path.abspath(config['ispec']) if 'ispec' in config else ''
        self.platform_spec = os.path.abspath(config['pspec']) if 'ispec' in config else ''
        self.make = config['make'] if 'make' in config else 'make'
        logger.debug("SAIL CSim plugin initialised using the following configuration.")
        for entry in config:
            logger.debug(entry + ' : ' + config[entry])
        return sclass

    def initialise(self, suite, work_dir, archtest_env):
        self.suite = suite
        self.work_dir = work_dir
        self.objdump_cmd = 'riscv{1}-unknown-linux-gnu-objdump -D {0} > {2};'
        self.compile_cmd = 'riscv{1}-unknown-linux-gnu-gcc -march={0} \
         -static -mcmodel=medany -fvisibility=hidden -nostdlib -nostartfiles\
         -T ' + self.pluginpath + '/env/link.ld\
         -I ' + self.pluginpath + '/env/\
         -I ' + archtest_env

    def build(self, isa_yaml, platform_yaml):
        ispec = utils.load_yaml(isa_yaml)['hart0']
        self.xlen = ('64' if 64 in ispec['supported_xlen'] else '32')
        self.isa = 'rv' + self.xlen
        self.compile_cmd = self.compile_cmd + ' -mabi=' + ('lp64 ' if 64 in ispec['supported_xlen'] else 'ilp32 ')
        if "G" in ispec["ISA"]:
            self.isa += "IMAFDZicsr_Zifencei"
        if "I" in ispec["ISA"]:
            self.isa += 'i'
        if "M" in ispec["ISA"]:
            self.isa += 'm'
        if "C" in ispec["ISA"]:
            self.isa += 'c'
        if "F" in ispec["ISA"]:
            self.isa += 'f'
        if "D" in ispec["ISA"]:
            self.isa += 'd'
        objdump = "riscv{0}-unknown-elf-objdump".format(self.xlen)
        if shutil.which(objdump) is None:
            logger.error(objdump + ": executable not found. Please check environment setup.")
            # raise SystemExit(1)
        compiler = "riscv{0}-unknown-elf-gcc".format(self.xlen)
        if shutil.which(compiler) is None:
            logger.error(compiler + ": executable not found. Please check environment setup.")
            # raise SystemExit(1)
        if shutil.which(self.sail_exe[self.xlen]) is None:
            logger.error(self.sail_exe[self.xlen] + ": executable not found. Please check environment setup.")
            # raise SystemExit(1)
        if shutil.which(self.make) is None:
            logger.error(self.make + ": executable not found. Please check environment setup.")
            # raise SystemExit(1)

    def runTests(self, testList, cgf_file=None):
        if os.path.exists(self.work_dir + "/Makefile." + self.name[:-1]):
            os.remove(self.work_dir + "/Makefile." + self.name[:-1])
        make = makeUtil(makefilePath=os.path.join(self.work_dir, "Makefile." + self.name[:-1]))
        make.makeCommand = self.make + ' -j' + self.num_jobs
        for file in testList:
            testentry = testList[file]
            test = testentry['test_path']
            test_dir = testentry['work_dir']
            test_name = test.rsplit('/', 1)[1][:-2]
            log_file = os.path.join(test_dir, test_name + '.log')

            elf_file = os.path.join(test_dir, 'ref.elf')
            disas_file = os.path.join(test_dir, 'ref.disass')

            cmd = self.compile_cmd.format(testentry['isa'].lower(), self.xlen) + ' ' + test + ' -o ' + elf_file
            compile_cmd = cmd + ' -D' + " -D".join(testentry['macros'])

            disas_cmd = self.objdump_cmd.format(elf_file, self.xlen, disas_file)
            sig_file = os.path.join(test_dir, self.name[:-1] + ".signature")

            sim_cmd = self.sail_exe[self.xlen] + ' --test-signature={0} {1} > {2} 2>&1;'.format(sig_file, elf_file,
                                                                                                log_file)
            cov_str = ' '
            for label in testentry['coverage_labels']:
                cov_str += ' -l ' + label

            if cgf_file is not None:
                coverage_file = os.path.join(test_dir, 'coverage.rpt')
                coverage_cmd = 'riscv_isac --verbose info coverage -d \
                        -t {0} --parser-name c_sail -o {5}  \
                        --sig-label begin_signature  end_signature \
                        --test-label rvtest_code_begin rvtest_code_end \
                        -e {4} -c {1} -x{2} {3};'.format(log_file, ' -c '.join(cgf_file), self.xlen, cov_str, elf_file, coverage_file)
            else:
                coverage_cmd = ''

            make.add_filetarget(compile_cmd, elf_file, test)
            make.add_filetarget(disas_cmd, disas_file, elf_file)
            make.add_filetarget(sim_cmd, sig_file, elf_file)
            if len(coverage_cmd) > 0:
                make.add_filetarget(coverage_cmd, sig_file, test)

            if self.target_run:
                make.add_phonytarget(sig_file + ' ' + disas_file)
            else:
                make.add_phonytarget(elf_file)

        make.execute_all(self.work_dir)
