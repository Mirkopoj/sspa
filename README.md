# sspa

Module test asistant

## Instalation 

Works on both dietpi and raspberry pi os.

```shell
wget https://raw.githubusercontent.com/Mirkopoj/sspa_installer_script/master/sspa_install.sh
chmod +x sspa_install.sh
./sspa_install.sh
```

## Usage

```
sspa
sspa [OPTION]...

OPTIONS:
	-h --help		Prints this page and exit
	-u --update		Updates binaries and exit
	-v --verbose		Explain what is being done
	-q --quiet		Do no log to stdout, will overwrite --verbose
   -l --little-endian		Change net byte order from BigEndian to LittleEndian
   -H --hat		Change dac functionality to use software pwm as analog out
	-p --port		Especify a port for the TCP server to listen at, 8000 by default
	-V --version		Prints version information and exit

NOTE: you can uninstall the program at any time running:
	sspa_uninstall.sh
```
