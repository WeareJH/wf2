#!/bin/bash
#set -x #uncomment for debug mode

clear_term() {
	#comment out for output
	clear 
	#echo "debug -- clear"
}

aborting_install() {
	clear_term
	echo "Aborting Install"
}

path_check() {
	path_check_var=$(which wf2)
	if [ -x "$path_check_var" ] ; then
			echo "WF2 appears to be in your path! Continuing..."
			sleep 2
	else 
			echo "WF2 does not appear to be in your path,"
			echo "Would you like the installer to try to auto add WF2 to your path? (y/n) "
			read addToPath
			if [[ $addToPath == "y" ]] ; then
					touch ~/.zshrc
					echo "export PATH=\"$PATH:/opt\"" >> ~/.zshrc
					echo "Now we are going to refresh your terminal"
					sleep 2
					source ~/.zshrc
			else
					echo "Skipping add to path step"
			fi
	fi
}

exec_install() {
	clear_term
	echo "Beginning express download of WF2"
	curl -L "https://github.com/wearejh/wf2/releases/download/v0.18.0/wf2" --output wf2-temp-binary-file
	chmod +x ./wf2-temp-binary-file
	echo "Download successful!"
	echo "You may now be asked for your password to install the WF2 binary"
	sudo mkdir -p /opt
	sudo chown -R $(whoami) /opt
	mv ./wf2-temp-binary-file /opt/wf2
	path_check
	echo "Now the self-update function will run to get the latest version!"
	sleep 2
	wf2 self-update
	echo "Thank you for using the express installer!"
	echo "(You may need to run \"source ~/.zshrc\" - without the quotes - to see wf2)"
}

clear_term
echo "Welcome to the WF2 Express installer!"
echo "Would you like to install WF2? (y/n) "
read continueInstall
if [[ $continueInstall == "y" ]] ; then
		echo "Ok, installing now"
		path_to_executable=$(which wf2)
		if [ -x "$path_to_executable" ] ; then
				clear_term
				echo "Looks like wf2 is already installed"
				echo "Would you like to reinstall?"
			    echo " - this will delete your existing installation (y/n) "
				read reinstall
				if [[ $reinstall == "y" ]] ; then
						exec_install
				else
						aborting_install
				fi
		else
				exec_install
		fi
else
		aborting_install
fi

