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
	-p --port		Especify a port for the TCP server to listen at, 8000 by default
	-V --version		Prints version information and exit

NOTE: you can uninstall the program at any time running:
	sspa_uninstall.sh
```
