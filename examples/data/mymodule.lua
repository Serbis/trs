local mymodule = {}

function mymodule.foo()
    print("Hello World!")
end

function mymodule.split(self, inputstr, sep)
    if sep == nil then
        sep = "%s"
    end
    local t={}
    for str in string.gmatch(inputstr, "([^"..sep.."]+)") do
        table.insert(t, str)
    end
    return t
end

return mymodule