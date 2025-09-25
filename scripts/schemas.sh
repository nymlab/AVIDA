
for dir in $PWD/contracts/sdjwt-verifier/ $PWD/contracts/avida_example; do
 cd $dir
 cargo schema
 cd -
done
