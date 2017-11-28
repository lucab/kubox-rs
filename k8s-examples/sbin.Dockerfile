FROM lucab/kubox-base-amd64:dev-latest
COPY sbin /sbin/
USER 0
CMD ["/sbin/kubox"] 
