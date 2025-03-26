#!/bin/csh -f

setenv EPOVSN 4.0.0
setenv MYDIR $PWD/
setenv EPO /home/hep/HEP/epos${EPOVSN}/

# echo "Installation beginning, EPOS version ${EPOVSN}"
echo "Currently in directory ${MYDIR}, EPO is set ${EPO}"

setenv LIBDIR ${EPO}bin
setenv OPT ${MYDIR}options/
setenv HTO ${MYDIR}hists/
setenv CHK ${MYDIR}results/

echo "LIBDIR is set to  ${LIBDIR}"
echo "OPT is set to  ${OPT}"
echo "HTO is set to  ${HTO}"
echo "CHK is set to  ${CHK}"

cd $MYDIR
# tar xvfz epos${EPOVSN}.tgz

setenv ROOTSYS /home/hep/HEP/root
setenv FASTSYS /home/hep/HEP/fastjet-install
set path= ( $ROOTSYS/bin $FASTSYS/bin $path )

echo "ROOTSYS is set to  ${ROOTSYS}"
echo "FASTSYS is set to  ${FASTSYS}"
echo "path is set to ${path}"


setenv COP BASIC
setenv LD_LIBRARY_PATH $ROOTSYS/lib:$ROOTSYS/lib/root:/usr/local/lib
echo "LD_LIBRARY_PATH is set to  ${LD_LIBRARY_PATH}"
# cd $EPO
# rm -Rf $LIBDIR && cmake -B$LIBDIR && make -C$LIBDIR -j8
# cd examples/
cd $OPT
$EPO/scripts/epos pp7
