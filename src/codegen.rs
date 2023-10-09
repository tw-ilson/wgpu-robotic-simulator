struct CodeGen  {
    header: String,
    kernel_prefix: String,
    buffer_prefix: String,
    buffer_suffix: String,
    sharedmem_prefix: String,
    gid: Vec<String>,
    lid: Vec<String>,
    extra_args: Vec<String>,
    float4: Option<String>,
    half_prekernel: Option<String>,
    uses_vload:bool,
}
trait CodeGen {
    fn write_kernel(&self) -> String;
}
