value_type evaluate_root_{{ev.size}}_{{ev.begin}}_{{ev.end}}_{{ev.distance}}(
        std::array<value_type, {{ev.size}}> &a)
{ {% for stride in ev.strides %}
    for(std::size_t i = {{ev.begin}}; i < {{ev.end}}; i += {{2*stride}}) {
        a[i] = hash<{{hash_function}}>(a[i], a[i+{{stride}}]);
    }{% endfor %}
    return a[{{ev.begin}}];
}

