FROM lucab/kubox-base-amd4:dev-latest
COPY sbin /sbin/
USER 0
CMD ["/sbin/kubox"] 
