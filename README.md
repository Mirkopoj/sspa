# sspa

Automatic board tester

## Instalation 

```shell
wget https://raw.githubusercontent.com/Mirkopoj/sspa_installer_script/master/sspa_install.sh
chmod +x sspa_install.sh
./sspa_install.sh
```

## Usage

```
sspa
sspa [OPTION]...
sspa [OPTION]... [FILE]...

OPTIONS:
	-h --help		Prints this page and exit
	-u --update		Updates binaries and exit
	-v --verbose		Explain what is being done
	-q --quiet		Do no log to stdout, will overwrite --verbose
	-l --logfile <FILE>	Logs output to specified file, not afected by --quiet
	-V --version		Prints version information and exit

NOTE: you can uninstall the program at any time running:
	sspa_uninstall.sh
```
