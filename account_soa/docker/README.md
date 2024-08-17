

 - `docker run [OPTIONS] IMAGE[:TAG|@DIGEST] [COMMAND] [ARG...]`
 - `docker run mvv_rust_account_soa /bin/bash -it`
 - `docker run -it mvv_rust_account_soa /bin/bash` # hm... Does not work for this image ?!?!
 - `docker run --name test333 -it --rm --entrypoint="/bin/bash" mvv_rust_account_soa-debug-local`


Clean
 - Images `docker rmi $(docker images -q)`
 - Volumes `docker volume rm $(docker volume ls -qf dangling=true)`
 - MEGA-SUPER clean `docker system prune`
