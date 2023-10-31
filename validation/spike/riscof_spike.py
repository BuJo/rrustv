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


class spike(pluginTemplate):
    __model__ = "spike"
    __version__ = "XXX"

    def __init__(self, *args, **kwargs):
        sclass = super().__init__(*args, **kwargs)

        config = kwargs.get('config')

        self.ref_exe = os.path.join(config['PATH'] if 'PATH' in config else "", "spike")
        self.num_jobs = str(config['jobs'] if 'jobs' in config else 1)
        self.pluginpath = os.path.abspath(config['pluginpath'])
        self.isa_spec = os.path.abspath(config['ispec']) if 'ispec' in config else ''
        self.platform_spec = os.path.abspath(config['pspec']) if 'ispec' in config else ''
        self.make = config['make'] if 'make' in config else 'make'
        # We capture if the user would like the run the tests on the target or
        # not. If you are interested in just compiling the tests and not running
        # them on the target, then following variable should be set to False
        if 'target_run' in config and config['target_run'] == '0':
            self.target_run = False
        else:
            self.target_run = True
        logger.debug("spike plugin initialised using the following configuration.")
        for entry in config:
            logger.debug(entry + ' : ' + config[entry])
        return sclass

    def initialise(self, suite, work_dir, archtest_env):
        self.suite = suite
        if shutil.which(self.ref_exe) is None:
            logger.error('Please install Executable for DUTNAME to proceed further')
            raise SystemExit(1)
        self.work_dir = work_dir

        # TODO: The following assumes you are using the riscv-gcc toolchain. If
        #      not please change appropriately
        self.objdump_cmd = 'riscv{1}-unknown-elf-objdump -D {0} > {2};'
        self.compile_cmd = 'riscv{1}-unknown-elf-gcc -march={0} \
         -static -mcmodel=medany -fvisibility=hidden -nostdlib -nostartfiles\
         -T ' + self.pluginpath + '/env/link.ld\
         -I ' + self.pluginpath + '/env/\
         -I ' + archtest_env

        # set all the necessary variables like compile command, elf2hex
        # commands, objdump cmds. etc whichever you feel necessary and required
        # for your plugin.

    def build(self, isa_yaml, platform_yaml):
        ispec = utils.load_yaml(isa_yaml)['hart0']
        self.xlen = ('64' if 64 in ispec['supported_xlen'] else '32')
        # TODO: The following assumes you are using the riscv-gcc toolchain. If
        #      not please change appropriately
        self.compile_cmd = self.compile_cmd + ' -mabi=' + ('lp64 ' if 64 in ispec['supported_xlen'] else 'ilp32 ')
        self.isa = 'rv' + self.xlen
        if "G" in ispec["ISA"]:
            self.isa += "IMAFDZicsr_Zifencei"
        if "I" in ispec["ISA"]:
            self.isa += 'i'
        if "M" in ispec["ISA"]:
            self.isa += 'm'
        if "F" in ispec["ISA"]:
            self.isa += 'f'
        if "D" in ispec["ISA"]:
            self.isa += 'd'
        if "C" in ispec["ISA"]:
            self.isa += 'c'
        if "Zicsr" in ispec["ISA"]:
            self.isa += '_Zicsr'
        if "Zifencei" in ispec["ISA"]:
            self.isa += '_Zifencei'
        if "Zba" in ispec["ISA"]:
            self.isa += '_Zba'
        if "Zbb" in ispec["ISA"]:
            self.isa += '_Zbb'
        if "Zbc" in ispec["ISA"]:
            self.isa += '_Zbc'
        if "Zbkb" in ispec["ISA"]:
            self.isa += '_Zbkb'
        if "Zbkc" in ispec["ISA"]:
            self.isa += '_Zbkc'
        if "Zbkx" in ispec["ISA"]:
            self.isa += '_Zbkx'
        if "Zbs" in ispec["ISA"]:
            self.isa += '_Zbs'
        if "Zknd" in ispec["ISA"]:
            self.isa += '_Zknd'
        if "Zkne" in ispec["ISA"]:
            self.isa += '_Zkne'
        if "Zknh" in ispec["ISA"]:
            self.isa += '_Zknh'
        if "Zksed" in ispec["ISA"]:
            self.isa += '_Zksed'
        if "Zksh" in ispec["ISA"]:
            self.isa += '_Zksh'

        # based on the validated isa and platform configure your simulator or
        # build your RTL here

    def runTests(self, testList, cgf_file=None):
        if os.path.exists(self.work_dir + "/Makefile." + self.name[:-1]):
            os.remove(self.work_dir + "/Makefile." + self.name[:-1])
        make = makeUtil(makefilePath=os.path.join(self.work_dir, "Makefile." + self.name[:-1]))
        make.makeCommand = self.make + ' -j' + self.num_jobs
        for file in testList:
            testentry = testList[file]
            test = testentry['test_path']
            test_dir = testentry['work_dir']

            elf_file = os.path.join(test_dir, 'ref.elf')
            disas_file = os.path.join(test_dir, 'ref.disass')

            cmd = self.compile_cmd.format(testentry['isa'].lower(), self.xlen) + ' ' + test + ' -o ' + elf_file

            compile_cmd = cmd + ' -D' + " -D".join(testentry['macros'])

            disas_cmd = self.objdump_cmd.format(elf_file, self.xlen, disas_file)
            sig_file = os.path.join(test_dir, self.name[:-1] + ".signature")

            sim_cmd = self.ref_exe + ' --isa={0} +signature={1} +signature-granularity=4 {2}'.format(self.isa,
                                                                                                        sig_file, elf_file)

            make.add_filetarget(compile_cmd, elf_file, test)
            make.add_filetarget(disas_cmd, disas_file, elf_file)
            make.add_filetarget(sim_cmd, sig_file, elf_file)

            if self.target_run:
                make.add_phonytarget(sig_file + ' ' + disas_file)
            else:
                make.add_phonytarget(os.path.join(test_dir, elf_file))
        make.execute_all(self.work_dir)
