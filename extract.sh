fusermount -uz ./mount
rm ./mount -rf
mkdir ./mount

./save3ds_fuse --movable ./keys/movable.sed --boot9 ./keys/boot9.bin --bare ./test.cecd ./mount -v --extract

