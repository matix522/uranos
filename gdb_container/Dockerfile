FROM alpine:3.10.3

RUN apk update && apk add git \
                      python3 \
                      python3-dev \
                      gcc \
                      g++ \
                      guile \
                      readline-dev \
                      texinfo \
                      guile-libs \
                      guile-dev \
                      guile-doc \
                      gdb \
                      xz \
                      curl \
                      make \
                      libc-dev


RUN cd /tmp \
    && curl http://ftp.gnu.org/gnu/gdb/gdb-8.3.tar.xz --output gdb-8.3.tar.xz \
    && tar xvf gdb-8.3.tar.xz \
    && cd gdb-8.3 \
    && ./configure \ 
 	 	 --target=aarch64-none-elf \ 
 	 	 --prefix=/usr \ 
 	 	 --enable-languages=c,c++ \ 
 	 	 --disable-multilib \ 
 	 	 --enable-interwork \ 
 	 	 --with-system-readline \ 
 	 	 --disable-nls \ 
 	 	 --with-python=/usr/bin/python3.7 \ 
 	 	 --without-guile \ 
 	 	 --with-system-gdbinit=/etc/gdb/gdbinit \
    && make \
    && make install 

